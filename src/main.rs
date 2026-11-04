//! obs-lightweight-recorder - Ultra-lightweight Windows game recorder
//!
//! Usage:
<<<<<<< HEAD
<<<<<<< HEAD
=======
>>>>>>> origin/master
//!   recorder capture <output.mp4> [--crf 21] [--preset veryfast] [--profile balanced]
//!
//! Captures the screen via Windows Graphics Capture (WGC),
//! encodes with H.264 via ffmpeg/libx264,
<<<<<<< HEAD
=======
//!   recorder capture <output.mp4> [--crf 21] [--preset veryfast] [--profile balanced] [--encoder libx264]
//!
//! Captures the screen via Windows Graphics Capture (WGC),
//! encodes with H.264 via ffmpeg (libx264, nvenc, amf, or qsv),
>>>>>>> b7baee3 (Milestone 6)
=======
>>>>>>> origin/master
//! and muxes to MP4.

mod capture;
mod encode;
mod hardware;
mod metrics;
mod mux;

use anyhow::{Context, Result};
use clap::Parser;
<<<<<<< HEAD
<<<<<<< HEAD
use std::time::Duration;

use mux::{EncodingConfig, EncodingPreset, EncodingProfile};
=======
use std::time::{Duration, Instant};

use hardware::HardwareInfo;
use mux::{EncodingConfig, EncodingPreset, EncodingProfile, EncoderType};
use metrics::Metrics;
>>>>>>> b7baee3 (Milestone 6)
=======
use std::time::Duration;

use mux::{EncodingConfig, EncodingPreset, EncodingProfile};
>>>>>>> origin/master

/// Ultra-lightweight Windows game recorder.
#[derive(Parser, Debug)]
#[command(name = "recorder")]
#[command(about = "Minimal screen recorder for low-end Windows laptops")]
struct Cli {
    /// Command to run
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, clap::Subcommand)]
enum Commands {
    /// Start recording screen to MP4 file
    Capture {
        /// Output MP4 file path
        #[arg(value_name = "OUTPUT.mp4")]
        output: String,

        /// Recording duration in seconds (0 = unlimited until Ctrl+C)
        #[arg(long, default_value_t = 0)]
        duration: u64,

        /// Capture FPS (default: 30)
        #[arg(long, default_value_t = 30)]
        fps: u32,

        /// Width override (0 = use display resolution)
        #[arg(long, default_value_t = 0)]
        width: u32,

        /// Height override (0 = use display resolution)
        #[arg(long, default_value_t = 0)]
        height: u32,

        /// Window title substring to capture (game-specific window capture)
        #[arg(long)]
        window_title: Option<String>,

<<<<<<< HEAD
<<<<<<< HEAD
=======
>>>>>>> origin/master
        /// Constant Rate Factor (0-51, lower = higher quality, default: 21 for balanced profile)
        #[arg(long)]
        crf: Option<u32>,

<<<<<<< HEAD
=======
        /// Encoder: auto, libx264, nvenc, amf, qsv (default: auto)
        #[arg(long, default_value_t = String::from("auto"))]
        encoder: String,

        /// Constant Rate Factor (0-51, lower = higher quality, libx264 only)
        #[arg(long)]
        crf: Option<u32>,

        /// Quality level for hardware encoders (encoder-specific scale)
        #[arg(long)]
        quality: Option<u32>,

>>>>>>> b7baee3 (Milestone 6)
=======
>>>>>>> origin/master
        /// x264 encoding preset (default: veryfast for balanced profile)
        #[arg(long)]
        preset: Option<String>,

        /// Encoding profile: quality, balanced, small (default: balanced)
        #[arg(long, default_value_t = String::from("balanced"))]
        profile: String,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {:?}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Capture {
            output,
            duration,
            fps,
            width,
            height,
            window_title,
<<<<<<< HEAD
<<<<<<< HEAD
            crf,
=======
            encoder,
            crf,
            quality,
>>>>>>> b7baee3 (Milestone 6)
=======
            crf,
>>>>>>> origin/master
            preset,
            profile,
        } => {
            capture_screen(
<<<<<<< HEAD
<<<<<<< HEAD
                &output, duration, fps, width, height, window_title.as_deref(),
                crf, preset, &profile,
=======
                &output,
                duration,
                fps,
                width,
                height,
                window_title.as_deref(),
                &encoder,
                crf,
                quality,
                preset,
                &profile,
>>>>>>> b7baee3 (Milestone 6)
=======
                &output, duration, fps, width, height, window_title.as_deref(),
                crf, preset, &profile,
>>>>>>> origin/master
            )
        }
    }
}

fn capture_screen(
    output: &str,
    duration_secs: u64,
    fps: u32,
    width: u32,
    height: u32,
    window_title: Option<&str>,
<<<<<<< HEAD
<<<<<<< HEAD
    crf: Option<u32>,
=======
    encoder_name: &str,
    crf: Option<u32>,
    quality: Option<u32>,
>>>>>>> b7baee3 (Milestone 6)
=======
    crf: Option<u32>,
>>>>>>> origin/master
    preset: Option<String>,
    profile_name: &str,
) -> Result<()> {
    // ── Detect hardware ──────────────────────────────────────────
<<<<<<< HEAD
<<<<<<< HEAD
    let hw = hardware::HardwareInfo::detect()
        .context("Failed to detect hardware")?;
=======
    let hw = HardwareInfo::detect().context("Failed to detect hardware")?;
>>>>>>> b7baee3 (Milestone 6)
=======
    let hw = hardware::HardwareInfo::detect()
        .context("Failed to detect hardware")?;
>>>>>>> origin/master
    hw.print_summary();

    // Get recommended configuration from hardware detection
    let recommended = hw.recommend_config();
    eprintln!(
<<<<<<< HEAD
<<<<<<< HEAD
=======
>>>>>>> origin/master
        "Auto-config: profile={} encoder={} crf={} preset={} reason: {}",
        recommended.profile.as_str(),
        recommended.encoder,
        recommended.crf,
<<<<<<< HEAD
=======
        "Auto-config: profile={} encoder={} crf={} quality={} preset={} reason: {}",
        recommended.profile.as_str(),
        recommended.encoder,
        recommended.crf,
        recommended.quality_level,
>>>>>>> b7baee3 (Milestone 6)
=======
>>>>>>> origin/master
        recommended.preset,
        recommended.reason
    );
    eprintln!("");

<<<<<<< HEAD
<<<<<<< HEAD
    // Resolve profile: CLI override takes precedence over recommendation
    let profile = if profile_name != "balanced" {
        // User explicitly specified a profile
=======
    // Resolve encoder: CLI override takes precedence over recommendation.
    // If user explicitly requests an encoder other than "auto", validate it
    // is actually available before starting recording.
    let encoder_type = if encoder_name != "auto" {
        let et = EncoderType::from_str(encoder_name)
            .with_context(|| format!("Invalid encoder: {}", encoder_name))?;
        // Validate explicit hardware encoder requests are available.
        match et {
            EncoderType::Nvenc if !hw.nvenc_available => {
                anyhow::bail!(
                    "Encoder 'nvenc' requested but NVIDIA NVENC is not available on this system. \
                     No compatible NVIDIA GPU or driver was detected. Use --encoder libx264 or --encoder auto instead."
                );
            }
            EncoderType::Amf if !hw.amf_available => {
                anyhow::bail!(
                    "Encoder 'amf' requested but AMD AMF is not available on this system. \
                     No compatible AMD GPU or driver was detected. Use --encoder libx264 or --encoder auto instead."
                );
            }
            EncoderType::Qsv if !hw.quicksync_available => {
                anyhow::bail!(
                    "Encoder 'qsv' requested but Intel Quick Sync is not available on this system. \
                     No compatible Intel GPU or driver was detected. Use --encoder libx264 or --encoder auto instead."
                );
            }
            _ => et,
        }
    } else {
        match recommended.encoder.as_str() {
            "nvenc" => EncoderType::Nvenc,
            "amf" => EncoderType::Amf,
            "qsv" => EncoderType::Qsv,
            _ => EncoderType::Libx264,
        }
    };

    // Resolve profile: CLI override takes precedence over recommendation
    let profile = if profile_name != "balanced" {
>>>>>>> b7baee3 (Milestone 6)
=======
    // Resolve profile: CLI override takes precedence over recommendation
    let profile = if profile_name != "balanced" {
        // User explicitly specified a profile
>>>>>>> origin/master
        EncodingProfile::from_str(profile_name)
            .with_context(|| format!("Invalid profile: {}", profile_name))?
    } else {
        match recommended.profile {
            hardware::RecommendedProfile::Performance => EncodingProfile::Small,
            hardware::RecommendedProfile::Balanced => EncodingProfile::Balanced,
            hardware::RecommendedProfile::Quality => EncodingProfile::Quality,
        }
    };

    // Resolve preset: CLI override > recommended > profile default
    let encoding_preset = match &preset {
        Some(p) => EncodingPreset::from_str(p)
            .with_context(|| format!("Invalid preset: {}", p))?,
        None => EncodingPreset::from_str(&recommended.preset)
            .with_context(|| format!("Invalid recommended preset: {}", recommended.preset))?,
    };

<<<<<<< HEAD
<<<<<<< HEAD
    // Resolve CRF: CLI override > recommended > profile default
    let crf = crf.unwrap_or(recommended.crf);
=======
    // Resolve CRF/quality: CLI override > recommended > profile default
    let crf = crf.unwrap_or(recommended.crf);
    let quality_level = quality.unwrap_or(recommended.quality_level);
>>>>>>> b7baee3 (Milestone 6)
=======
    // Resolve CRF: CLI override > recommended > profile default
    let crf = crf.unwrap_or(recommended.crf);
>>>>>>> origin/master

    // Resolve FPS: CLI value (has default of 30) > recommended
    let out_fps = if fps != 30 {
        fps
    } else {
        recommended.fps
    };

    // Initialize capture: window-specific or monitor (desktop)
    let mut cap: capture::Capture;
    let (capture_width, capture_height): (u32, u32);
    if let Some(title) = window_title {
        let hwnd = capture::find_window_by_title(title)
            .context("Failed to find window by title")?;
        eprintln!("Window capture: HWND {:?}", hwnd);
        cap = capture::Capture::from_window(hwnd)
            .context("Failed to initialize WGC window capture")?;
    } else {
        cap = capture::Capture::from_monitor()
            .context("Failed to initialize WGC capture")?;
    }
    capture_width = cap.width();
    capture_height = cap.height();

    // Resolve output dimensions: CLI override if non-zero, otherwise capture resolution
<<<<<<< HEAD
<<<<<<< HEAD
    // (which may be overridden by recommended config)
=======
>>>>>>> b7baee3 (Milestone 6)
=======
    // (which may be overridden by recommended config)
>>>>>>> origin/master
    let out_width = if width > 0 {
        width
    } else {
        recommended.width.max(capture_width)
    };
    let out_height = if height > 0 {
        height
    } else {
        recommended.height.max(capture_height)
    };

    eprintln!("obs-lightweight-recorder v0.1.0");
    eprintln!("Output: {}", output);
<<<<<<< HEAD
<<<<<<< HEAD
    eprintln!("Profile: {:?}", profile);
    eprintln!("CRF: {}", crf);
=======
    eprintln!("Encoder: {}", encoder_type.display_name());
    eprintln!("Profile: {:?}", profile);
    eprintln!("CRF: {}", crf);
    eprintln!("Quality: {}", quality_level);
>>>>>>> b7baee3 (Milestone 6)
=======
    eprintln!("Profile: {:?}", profile);
    eprintln!("CRF: {}", crf);
>>>>>>> origin/master
    eprintln!("Preset: {}", encoding_preset.as_str());
    eprintln!("FPS: {}", out_fps);
    eprintln!("Resolution: {}x{}", out_width, out_height);
    eprintln!("Duration: {}s (0 = unlimited)", duration_secs);
    eprintln!("");
<<<<<<< HEAD
<<<<<<< HEAD
=======
>>>>>>> origin/master

    let config = EncodingConfig::new(
        profile,
        Some(crf),
<<<<<<< HEAD
=======
    eprintln!("Recording... Press Ctrl+C to stop.");
    eprintln!("");

    let config = EncodingConfig::new(
        profile,
        encoder_type,
        Some(crf),
        Some(quality_level),
>>>>>>> b7baee3 (Milestone 6)
=======
>>>>>>> origin/master
        Some(encoding_preset),
        out_width,
        out_height,
        out_fps,
    )?;

    eprintln!(
        "Capture initialized: {}x{} @ {}fps",
        cap.width(), cap.height(), config.fps
    );

    // Initialize encoder
<<<<<<< HEAD
<<<<<<< HEAD
    let mut encoder = encode::Encoder::new(config.width, config.height, config.fps)
        .context("Failed to initialize encoder")?;
=======
    let mut encoder = encode::Encoder::new(
        config.width,
        config.height,
        config.fps,
        config.encoder_type,
    )
    .context("Failed to initialize encoder")?;
>>>>>>> b7baee3 (Milestone 6)
=======
    let mut encoder = encode::Encoder::new(config.width, config.height, config.fps)
        .context("Failed to initialize encoder")?;
>>>>>>> origin/master
    eprintln!("Encoder initialized: {}", encoder.name());

    // Initialize muxer
    let mut muxer = mux::Muxer::new(output, &config)
        .context("Failed to initialize muxer")?;
    eprintln!("Muxer initialized: {}", output);
<<<<<<< HEAD
<<<<<<< HEAD
=======
>>>>>>> origin/master
    eprintln!("");
    eprintln!("Recording... Press Ctrl+C to stop.");
    eprintln!("");

    let frame_interval = Duration::from_nanos(1_000_000_000 / config.fps as u64);
    let mut last_frame_time = std::time::Instant::now();
    let start_time = std::time::Instant::now();
    let mut dropped = 0u64;

    // Write the first captured frame immediately to avoid ffmpeg stdin pipe
    // race condition on Windows (ffmpeg times out waiting for input).
    match cap.next_frame() {
        Ok(Some(frame)) => {
            match encoder.encode_frame(&frame) {
                Ok(Some(packet)) => {
                    if let Err(e) = muxer.write_video(&packet) {
                        eprintln!("Mux error (first frame): {:?}", e);
                        dropped += 1;
<<<<<<< HEAD
=======

    // Initialize metrics
    let mut metrics = Metrics::new();

    let frame_interval = Duration::from_nanos(1_000_000_000 / config.fps as u64);
    let mut last_frame_time = Instant::now();
    let start_time = Instant::now();

    // Write the first captured frame immediately to avoid ffmpeg stdin pipe
    // race condition on Windows (ffmpeg times out waiting for input).
    let encode_start = Instant::now();
    match cap.next_frame() {
        Ok(Some(frame)) => {
            metrics.frame_captured();
            match encoder.encode_frame(&frame) {
                Ok(Some(packet)) => {
                    let encode_elapsed = encode_start.elapsed().as_nanos() as u64;
                    metrics.frame_encoded(encode_elapsed);
                    if let Err(e) = muxer.write_video(&packet) {
                        eprintln!("Mux error (first frame): {:?}", e);
                        metrics.frame_dropped();
>>>>>>> b7baee3 (Milestone 6)
=======
>>>>>>> origin/master
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    eprintln!("Encode error (first frame): {:?}", e);
<<<<<<< HEAD
<<<<<<< HEAD
                    dropped += 1;
=======
                    metrics.frame_dropped();
>>>>>>> b7baee3 (Milestone 6)
=======
                    dropped += 1;
>>>>>>> origin/master
                }
            }
        }
        Ok(None) => {
            eprintln!("Warning: no initial frame captured");
        }
        Err(e) => {
            eprintln!("Capture error (initial frame): {:?}", e);
        }
    }

    loop {
        // Check duration limit
        if duration_secs > 0 && start_time.elapsed().as_secs() >= duration_secs {
            eprintln!("\nDuration limit reached.");
            break;
        }

        // Frame rate limiting
<<<<<<< HEAD
<<<<<<< HEAD
        let now = std::time::Instant::now();
=======
        let now = Instant::now();
>>>>>>> b7baee3 (Milestone 6)
=======
        let now = std::time::Instant::now();
>>>>>>> origin/master
        if now.duration_since(last_frame_time) < frame_interval {
            std::thread::sleep(Duration::from_millis(1));
            continue;
        }
        last_frame_time = now;

        // Capture frame
<<<<<<< HEAD
<<<<<<< HEAD
=======
>>>>>>> origin/master
        match cap.next_frame() {
            Ok(Some(frame)) => {
                // Encode frame
                match encoder.encode_frame(&frame) {
                    Ok(Some(packet)) => {
                        // Mux packet
                        if let Err(e) = muxer.write_video(&packet) {
                            eprintln!("Mux error: {:?}", e);
                            dropped += 1;
<<<<<<< HEAD
=======
        let encode_start = Instant::now();
        match cap.next_frame() {
            Ok(Some(frame)) => {
                metrics.frame_captured();
                // Encode frame
                match encoder.encode_frame(&frame) {
                    Ok(Some(packet)) => {
                        let encode_elapsed = encode_start.elapsed().as_nanos() as u64;
                        metrics.frame_encoded(encode_elapsed);
                        // Mux packet
                        if let Err(e) = muxer.write_video(&packet) {
                            eprintln!("Mux error: {:?}", e);
                            metrics.frame_dropped();
>>>>>>> b7baee3 (Milestone 6)
=======
>>>>>>> origin/master
                        }
                    }
                    Ok(None) => {
                        // Encoder needs more frames or buffering - skip
                    }
                    Err(e) => {
                        eprintln!("Encode error: {:?}", e);
<<<<<<< HEAD
<<<<<<< HEAD
                        dropped += 1;
=======
                        metrics.frame_dropped();
>>>>>>> b7baee3 (Milestone 6)
=======
                        dropped += 1;
>>>>>>> origin/master
                    }
                }
            }
            Ok(None) => {
                // No frame ready yet - normal
            }
            Err(e) => {
                eprintln!("Capture error: {:?}", e);
<<<<<<< HEAD
<<<<<<< HEAD
=======
>>>>>>> origin/master
                dropped += 1;
            }
        }

        // Print stats every second
        let elapsed = start_time.elapsed();
        if elapsed.as_secs_f64().floor()
            != elapsed.saturating_sub(std::time::Duration::from_millis(500)).as_secs_f64().floor()
        {
            // Print stats
            let capture_fps = cap.fps();
            eprint!(
                "\rFrame: {} | CapFPS: {:.1} | EncFPS: {:.1} | Dropped: {}",
                cap.frame_count(),
                capture_fps,
                encoder.fps(),
                dropped
<<<<<<< HEAD
=======
                metrics.frame_dropped();
            }
        }

        // Print stats every second using metrics
        if metrics.should_print_stats() {
            metrics.mark_stats_printed();
            eprint!(
                "\rCaptured: {} | Encoded: {} | CapFPS: {:.1} | EncFPS: {:.1} | AvgEnc: {:.2}ms | Dropped: {}",
                metrics.captured(),
                metrics.encoded(),
                metrics.capture_fps(),
                metrics.encode_fps(),
                metrics.avg_encode_ms(),
                metrics.dropped()
>>>>>>> b7baee3 (Milestone 6)
=======
>>>>>>> origin/master
            );
        }
    }

    // Flush encoder
    eprintln!("\nFinalizing recording...");
    match encoder.finish() {
        Ok(packets) => {
            for packet in packets {
                let _ = muxer.write_video(&packet);
            }
        }
        Err(e) => eprintln!("Encoder flush error: {:?}", e),
    }

    // Close muxer
    if let Err(e) = muxer.finish() {
        eprintln!("Muxer finish error: {:?}", e);
    }

    // Stop capture
    cap.stop()?;

    let elapsed = start_time.elapsed().as_secs_f64();
<<<<<<< HEAD
<<<<<<< HEAD
=======
>>>>>>> origin/master
    eprintln!(
        "Recording complete. {} frames in {:.2}s ({:.1} fps avg)",
        cap.frame_count(),
        elapsed,
        cap.frame_count() as f64 / elapsed
    );

    Ok(())
<<<<<<< HEAD
}
=======
    eprintln!("");
    eprintln!("=== Recording Summary ===");
    eprintln!("Encoder: {}", encoder_type.display_name());
    eprintln!("Output: {}", output);
    eprintln!(
        "Frames captured: {} | Encoded: {} | Dropped: {}",
        metrics.captured(),
        metrics.encoded(),
        metrics.dropped()
    );
    eprintln!(
        "Capture FPS: {:.1} | Encode FPS: {:.1} | Avg encode: {:.2}ms | CPU: {:.1}%",
        metrics.capture_fps(),
        metrics.encode_fps(),
        metrics.avg_encode_ms(),
        metrics.cpu_utilization()
    );
    eprintln!("GPU utilization: not available (vendor-specific API required)");
    eprintln!("Duration: {:.2}s", elapsed);
    eprintln!(
        "Capture rate: {:.1} fps avg",
        metrics.captured() as f64 / elapsed
    );
    eprintln!("========================");

    Ok(())
}
>>>>>>> b7baee3 (Milestone 6)
=======
}
>>>>>>> origin/master
