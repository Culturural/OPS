//! obs-lightweight-recorder - Ultra-lightweight Windows game recorder
//!
//! Usage:
//!   recorder capture <output.mp4> [--crf 21] [--preset veryfast] [--profile balanced]
//!
//! Captures the screen via Windows Graphics Capture (WGC),
//! encodes with H.264 via ffmpeg/libx264,
//! and muxes to MP4.

mod capture;
mod encode;
mod hardware;
mod metrics;
mod mux;

use anyhow::{Context, Result};
use clap::Parser;
use std::time::Duration;

use mux::{EncodingConfig, EncodingPreset, EncodingProfile};

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

        /// Constant Rate Factor (0-51, lower = higher quality, default: 21 for balanced profile)
        #[arg(long)]
        crf: Option<u32>,

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
            crf,
            preset,
            profile,
        } => {
            capture_screen(
                &output, duration, fps, width, height, window_title.as_deref(),
                crf, preset, &profile,
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
    crf: Option<u32>,
    preset: Option<String>,
    profile_name: &str,
) -> Result<()> {
    // ── Detect hardware ──────────────────────────────────────────
    let hw = hardware::HardwareInfo::detect()
        .context("Failed to detect hardware")?;
    hw.print_summary();

    // Get recommended configuration from hardware detection
    let recommended = hw.recommend_config();
    eprintln!(
        "Auto-config: profile={} encoder={} crf={} preset={} reason: {}",
        recommended.profile.as_str(),
        recommended.encoder,
        recommended.crf,
        recommended.preset,
        recommended.reason
    );
    eprintln!("");

    // Resolve profile: CLI override takes precedence over recommendation
    let profile = if profile_name != "balanced" {
        // User explicitly specified a profile
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

    // Resolve CRF: CLI override > recommended > profile default
    let crf = crf.unwrap_or(recommended.crf);

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
    // (which may be overridden by recommended config)
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
    eprintln!("Profile: {:?}", profile);
    eprintln!("CRF: {}", crf);
    eprintln!("Preset: {}", encoding_preset.as_str());
    eprintln!("FPS: {}", out_fps);
    eprintln!("Resolution: {}x{}", out_width, out_height);
    eprintln!("Duration: {}s (0 = unlimited)", duration_secs);
    eprintln!("");

    let config = EncodingConfig::new(
        profile,
        Some(crf),
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
    let mut encoder = encode::Encoder::new(config.width, config.height, config.fps)
        .context("Failed to initialize encoder")?;
    eprintln!("Encoder initialized: {}", encoder.name());

    // Initialize muxer
    let mut muxer = mux::Muxer::new(output, &config)
        .context("Failed to initialize muxer")?;
    eprintln!("Muxer initialized: {}", output);
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
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    eprintln!("Encode error (first frame): {:?}", e);
                    dropped += 1;
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
        let now = std::time::Instant::now();
        if now.duration_since(last_frame_time) < frame_interval {
            std::thread::sleep(Duration::from_millis(1));
            continue;
        }
        last_frame_time = now;

        // Capture frame
        match cap.next_frame() {
            Ok(Some(frame)) => {
                // Encode frame
                match encoder.encode_frame(&frame) {
                    Ok(Some(packet)) => {
                        // Mux packet
                        if let Err(e) = muxer.write_video(&packet) {
                            eprintln!("Mux error: {:?}", e);
                            dropped += 1;
                        }
                    }
                    Ok(None) => {
                        // Encoder needs more frames or buffering - skip
                    }
                    Err(e) => {
                        eprintln!("Encode error: {:?}", e);
                        dropped += 1;
                    }
                }
            }
            Ok(None) => {
                // No frame ready yet - normal
            }
            Err(e) => {
                eprintln!("Capture error: {:?}", e);
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
    eprintln!(
        "Recording complete. {} frames in {:.2}s ({:.1} fps avg)",
        cap.frame_count(),
        elapsed,
        cap.frame_count() as f64 / elapsed
    );

    Ok(())
}