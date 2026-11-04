//! Tauri application entry point for OPS desktop app.
//!
//! Exposes Rust recording engine commands to the React frontend via Tauri IPC.
//! All capture/encode/mux operations run on a single dedicated thread to avoid
//! COM apartment threading issues with HWND/WGC.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use ops_engine::{
    capture::{Capture, enumerate_windows, find_window_by_title, WindowInfo},
    encode::Encoder,
    hardware::HardwareInfo,
    metrics::Metrics,
    mux::{EncodingConfig, EncodingPreset, EncodingProfile, EncoderType, Muxer},
};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::command;

// ─── Command DTOs ───────────────────────────────────────────────────────────

#[derive(Serialize, Clone)]
pub struct HardwareInfoDto {
    pub cpu_manufacturer: String,
    pub cpu_model: String,
    pub cpu_cores: u32,
    pub cpu_threads: u32,
    pub total_ram_mb: u64,
    pub gpu_manufacturer: Option<String>,
    pub gpu_model: Option<String>,
    pub gpu_adapter_count: u32,
    pub windows_version: String,
    pub windows_build: u32,
    pub display_width: u32,
    pub display_height: u32,
    pub hardware_encoder_available: bool,
    pub nvenc_available: bool,
    pub amf_available: bool,
    pub quicksync_available: bool,
}

impl From<&HardwareInfo> for HardwareInfoDto {
    fn from(hw: &HardwareInfo) -> Self {
        HardwareInfoDto {
            cpu_manufacturer: hw.cpu_manufacturer.clone(),
            cpu_model: hw.cpu_model.clone(),
            cpu_cores: hw.cpu_cores,
            cpu_threads: hw.cpu_threads,
            total_ram_mb: hw.total_ram_mb,
            gpu_manufacturer: hw.gpu_manufacturer.clone(),
            gpu_model: hw.gpu_model.clone(),
            gpu_adapter_count: hw.gpu_adapter_count,
            windows_version: hw.windows_version.clone(),
            windows_build: hw.windows_build,
            display_width: hw.display_width,
            display_height: hw.display_height,
            hardware_encoder_available: hw.hardware_encoder_available,
            nvenc_available: hw.nvenc_available,
            amf_available: hw.amf_available,
            quicksync_available: hw.quicksync_available,
        }
    }
}

#[derive(Serialize, Clone)]
pub struct WindowDto {
    pub hwnd: isize,
    pub title: String,
    pub process_id: u32,
}

impl From<&WindowInfo> for WindowDto {
    fn from(w: &WindowInfo) -> Self {
        WindowDto {
            hwnd: w.hwnd.0 as isize,
            title: w.title.clone(),
            process_id: w.process_id,
        }
    }
}

#[derive(Serialize, Clone)]
pub struct RecordingStatusDto {
    pub is_recording: bool,
    pub frames_captured: u64,
    pub frames_encoded: u64,
    pub frames_dropped: u64,
    pub capture_fps: f64,
    pub encode_fps: f64,
    pub avg_encode_ms: f64,
    pub cpu_utilization: f64,
    pub elapsed_seconds: f64,
    pub output_path: Option<String>,
    pub encoder: String,
}

#[derive(Deserialize, Clone)]
pub struct RecordingConfig {
    pub output_path: String,
    pub fps: u32,
    pub width: u32,
    pub height: u32,
    pub encoder: String,
    pub crf: Option<u32>,
    pub quality: Option<u32>,
    pub preset: Option<String>,
    pub profile: String,
    pub window_title: Option<String>,
}

#[derive(Serialize)]
pub struct StartRecordingResult {
    pub success: bool,
    pub message: String,
}

// ─── Global Recorder State ──────────────────────────────────────────────────

struct RecorderStatus {
    is_recording: AtomicBool,
    frames_captured: Mutex<u64>,
    frames_encoded: Mutex<u64>,
    frames_dropped: Mutex<u64>,
    total_encode_time_ns: Mutex<u64>,
    start_time: Mutex<Option<Instant>>,
    output_path: Mutex<Option<String>>,
    encoder_name: Mutex<String>,
}

impl Default for RecorderStatus {
    fn default() -> Self {
        Self {
            is_recording: AtomicBool::new(false),
            frames_captured: Mutex::new(0),
            frames_encoded: Mutex::new(0),
            frames_dropped: Mutex::new(0),
            total_encode_time_ns: Mutex::new(0),
            start_time: Mutex::new(None),
            output_path: Mutex::new(None),
            encoder_name: Mutex::new(String::new()),
        }
    }
}

static RECORDER_STATE: std::sync::OnceLock<Arc<Mutex<RecorderStatus>>> =
    std::sync::OnceLock::new();

fn recorder_state() -> Arc<Mutex<RecorderStatus>> {
    RECORDER_STATE
        .get_or_init(|| Arc::new(Mutex::new(RecorderStatus::default())))
        .clone()
}

// ─── Tauri Commands ─────────────────────────────────────────────────────────

#[command]
fn get_hardware_info() -> Result<HardwareInfoDto, String> {
    let hw = HardwareInfo::detect().map_err(|e| e.to_string())?;
    Ok(HardwareInfoDto::from(&hw))
}

#[command]
fn enumerate_windows_cmd() -> Result<Vec<WindowDto>, String> {
    let windows = enumerate_windows().map_err(|e| e.to_string())?;
    Ok(windows.iter().map(WindowDto::from).collect())
}

#[command]
fn get_recording_status() -> RecordingStatusDto {
    let state_arc = recorder_state();
    let inner = state_arc.lock().unwrap();

    let elapsed = {
        let start_guard = inner.start_time.lock().unwrap();
        start_guard.map(|t| t.elapsed().as_secs_f64()).unwrap_or(0.0)
    };

    let fc = *inner.frames_captured.lock().unwrap();
    let fe = *inner.frames_encoded.lock().unwrap();
    let fd = *inner.frames_dropped.lock().unwrap();
    let cet = *inner.total_encode_time_ns.lock().unwrap();

    let output = inner.output_path.lock().unwrap().clone();
    let encoder = inner.encoder_name.lock().unwrap().clone();
    let is_rec = inner.is_recording.load(Ordering::Relaxed);
    drop(inner);

    let capture_fps = if elapsed > 0.0 { fc as f64 / elapsed } else { 0.0 };
    let encode_fps = if elapsed > 0.0 { fe as f64 / elapsed } else { 0.0 };
    let avg_encode_ms = if fe > 0 { cet as f64 / fe as f64 / 1_000_000.0 } else { 0.0 };

    RecordingStatusDto {
        is_recording: is_rec,
        frames_captured: fc,
        frames_encoded: fe,
        frames_dropped: fd,
        capture_fps,
        encode_fps,
        avg_encode_ms,
        cpu_utilization: 0.0,
        elapsed_seconds: elapsed,
        output_path: output,
        encoder: encoder,
    }
}

#[command]
fn start_recording(config: RecordingConfig) -> Result<StartRecordingResult, String> {
    let state = recorder_state();
    {
        let inner = state.lock().unwrap();
        if inner.is_recording.load(Ordering::Relaxed) {
            return Ok(StartRecordingResult {
                success: false,
                message: "Already recording".to_string(),
            });
        }
    }

    // Detect hardware for auto-config
    let hw = HardwareInfo::detect().map_err(|e| e.to_string())?;
    let recommended = hw.recommend_config();

    // Resolve encoder
    let encoder_type = if config.encoder != "auto" {
        match ops_engine::mux::EncoderType::from_str(&config.encoder) {
            Ok(et) => {
                match et {
                    ops_engine::mux::EncoderType::Nvenc if !hw.nvenc_available => {
                        return Err("NVIDIA NVENC is not available on this system".to_string());
                    }
                    ops_engine::mux::EncoderType::Amf if !hw.amf_available => {
                        return Err("AMD AMF is not available on this system".to_string());
                    }
                    ops_engine::mux::EncoderType::Qsv if !hw.quicksync_available => {
                        return Err("Intel Quick Sync is not available on this system".to_string());
                    }
                    _ => et,
                }
            }
            Err(e) => return Err(format!("Invalid encoder: {}", e)),
        }
    } else {
        match recommended.encoder.as_str() {
            "nvenc" => ops_engine::mux::EncoderType::Nvenc,
            "amf" => ops_engine::mux::EncoderType::Amf,
            "qsv" => ops_engine::mux::EncoderType::Qsv,
            _ => ops_engine::mux::EncoderType::Libx264,
        }
    };

    // Resolve profile
    let profile = if config.profile != "balanced" {
        match ops_engine::mux::EncodingProfile::from_str(&config.profile) {
            Ok(p) => p,
            Err(e) => return Err(format!("Invalid profile: {}", e)),
        }
    } else {
        match recommended.profile {
            ops_engine::hardware::RecommendedProfile::Performance => {
                ops_engine::mux::EncodingProfile::Small
            }
            ops_engine::hardware::RecommendedProfile::Balanced => {
                ops_engine::mux::EncodingProfile::Balanced
            }
            ops_engine::hardware::RecommendedProfile::Quality => {
                ops_engine::mux::EncodingProfile::Quality
            }
        }
    };

    // Resolve preset
    let encoding_preset = match &config.preset {
        Some(p) => match ops_engine::mux::EncodingPreset::from_str(p) {
            Ok(pr) => pr,
            Err(e) => return Err(format!("Invalid preset: {}", e)),
        },
        None => match ops_engine::mux::EncodingPreset::from_str(&recommended.preset) {
            Ok(pr) => pr,
            Err(e) => return Err(format!("Invalid preset: {}", e)),
        },
    };

    let crf = config.crf.unwrap_or(recommended.crf);
    let quality_level = config.quality.unwrap_or(recommended.quality_level);
    let out_fps = if config.fps != 0 { config.fps } else { recommended.fps };

    // Reset status
    {
        let mut inner = state.lock().unwrap();
        *inner.frames_captured.lock().unwrap() = 0;
        *inner.frames_encoded.lock().unwrap() = 0;
        *inner.frames_dropped.lock().unwrap() = 0;
        *inner.total_encode_time_ns.lock().unwrap() = 0;
        *inner.start_time.lock().unwrap() = Some(Instant::now());
        *inner.output_path.lock().unwrap() = Some(config.output_path.clone());
        *inner.encoder_name.lock().unwrap() = encoder_type.display_name().to_string();
        inner.is_recording.store(true, Ordering::Relaxed);
    }

    let state = recorder_state();
    let config_clone = config;

    // Spawn recording thread - ALL capture/encode/mux happens here
    std::thread::spawn(move || {
        run_recording_thread(state, config_clone, profile, encoder_type, crf, quality_level, encoding_preset, out_fps);
    });

    Ok(StartRecordingResult {
        success: true,
        message: "Recording started".to_string(),
    })
}

fn run_recording_thread(
    state: Arc<Mutex<RecorderStatus>>,
    config: RecordingConfig,
    profile: EncodingProfile,
    encoder_type: EncoderType,
    crf: u32,
    quality_level: u32,
    preset: EncodingPreset,
    out_fps: u32,
) {
    // Initialize capture on this thread (COM apartment)
    let mut capture: Capture;
    if let Some(ref title) = config.window_title {
        match find_window_by_title(title) {
            Ok(hwnd) => {
                capture = match Capture::from_window(hwnd) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Failed to initialize window capture: {}", e);
                        let _ = state.lock().unwrap().is_recording.store(false, Ordering::Relaxed);
                        return;
                    }
                };
            }
            Err(e) => {
                eprintln!("Failed to find window: {}", e);
                let _ = state.lock().unwrap().is_recording.store(false, Ordering::Relaxed);
                return;
            }
        }
    } else {
        capture = match Capture::from_monitor() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to initialize monitor capture: {}", e);
                let _ = state.lock().unwrap().is_recording.store(false, Ordering::Relaxed);
                return;
            }
        }
    }
    let capture_width = capture.width();
    let capture_height = capture.height();

    let out_width = if config.width > 0 { config.width } else { capture_width };
    let out_height = if config.height > 0 { config.height } else { capture_height };

    let config_enc = match EncodingConfig::new(
        profile,
        encoder_type,
        Some(crf),
        Some(quality_level),
        Some(preset),
        out_width,
        out_height,
        out_fps,
    ) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Encoding config error: {}", e);
            let _ = state.lock().unwrap().is_recording.store(false, Ordering::Relaxed);
            return;
        }
    };

    let mut encoder = match Encoder::new(config_enc.width, config_enc.height, config_enc.fps, config_enc.encoder_type) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Encoder init error: {}", e);
            let _ = state.lock().unwrap().is_recording.store(false, Ordering::Relaxed);
            return;
        }
    };

    let mut muxer = match Muxer::new(&config.output_path, &config_enc) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Muxer init error: {}", e);
            let _ = state.lock().unwrap().is_recording.store(false, Ordering::Relaxed);
            return;
        }
    };

    // Write first frame immediately
    if let Ok(Some(frame)) = capture.next_frame() {
        *state.lock().unwrap().frames_captured.lock().unwrap() += 1;
        if let Ok(Some(packet)) = encoder.encode_frame(&frame) {
            *state.lock().unwrap().frames_encoded.lock().unwrap() += 1;
            if let Err(e) = muxer.write_video(&packet) {
                *state.lock().unwrap().frames_dropped.lock().unwrap() += 1;
                eprintln!("Mux error: {:?}", e);
            }
        }
    }

    let frame_interval = std::time::Duration::from_nanos(1_000_000_000 / out_fps as u64);

    while state.lock().unwrap().is_recording.load(Ordering::Relaxed) {
        std::thread::sleep(frame_interval);

        match capture.next_frame() {
            Ok(Some(frame)) => {
                *state.lock().unwrap().frames_captured.lock().unwrap() += 1;
                match encoder.encode_frame(&frame) {
                    Ok(Some(packet)) => {
                        *state.lock().unwrap().frames_encoded.lock().unwrap() += 1;
                        if let Err(e) = muxer.write_video(&packet) {
                            *state.lock().unwrap().frames_dropped.lock().unwrap() += 1;
                            eprintln!("Mux error: {:?}", e);
                        }
                    }
                    Ok(None) => {}
                    Err(e) => {
                        *state.lock().unwrap().frames_dropped.lock().unwrap() += 1;
                        eprintln!("Encode error: {:?}", e);
                    }
                }
            }
            Ok(None) => {}
            Err(e) => {
                *state.lock().unwrap().frames_dropped.lock().unwrap() += 1;
                eprintln!("Capture error: {:?}", e);
            }
        }
    }

    // Finalize
    if let Ok(packets) = encoder.finish() {
        for packet in packets {
            let _ = muxer.write_video(&packet);
        }
    }
    let _ = muxer.finish();
    let _ = capture.stop();

    state.lock().unwrap().is_recording.store(false, Ordering::Relaxed);
}

#[command]
fn stop_recording() -> Result<StartRecordingResult, String> {
    let state = recorder_state();
    let mut inner = state.lock().unwrap();

    if !inner.is_recording.load(Ordering::Relaxed) {
        return Ok(StartRecordingResult {
            success: false,
            message: "Not currently recording".to_string(),
        });
    }

    inner.is_recording.store(false, Ordering::Relaxed);

    Ok(StartRecordingResult {
        success: true,
        message: "Recording stopped".to_string(),
    })
}

#[command]
fn get_available_encoders() -> Result<Vec<String>, String> {
    Ok(vec![
        "auto".to_string(),
        "libx264".to_string(),
        "nvenc".to_string(),
        "amf".to_string(),
        "qsv".to_string(),
    ])
}

#[command]
fn get_available_profiles() -> Result<Vec<String>, String> {
    Ok(vec![
        "quality".to_string(),
        "balanced".to_string(),
        "small".to_string(),
    ])
}

#[command]
fn get_available_presets() -> Result<Vec<String>, String> {
    Ok(vec![
        "ultrafast".to_string(),
        "superfast".to_string(),
        "veryfast".to_string(),
        "faster".to_string(),
        "fast".to_string(),
        "medium".to_string(),
        "slow".to_string(),
        "slower".to_string(),
        "veryslow".to_string(),
        "placebo".to_string(),
    ])
}

// ─── Main ───────────────────────────────────────────────────────────────────

fn main() {
    tauri::Builder::<tauri::Wry>::default()
        .invoke_handler(tauri::generate_handler![
            get_hardware_info,
            enumerate_windows_cmd,
            get_recording_status,
            start_recording,
            stop_recording,
            get_available_encoders,
            get_available_profiles,
            get_available_presets,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
