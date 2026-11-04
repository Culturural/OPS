//! MP4 muxing module.
//!
//! Muxes captured video into an MP4 container by piping raw BGRA frames to an
//! `ffmpeg` subprocess, which performs H.264 encoding (libx264) and MP4 muxing,
//! producing a playable MP4 file.

use anyhow::{Context, Result};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

/// Encoding preset for x264.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodingPreset {
    Ultrafast,
    Superfast,
    Veryfast,
    Faster,
    Fast,
    Medium,
    Slow,
    Slower,
    Veryslow,
    Placebo,
}

impl EncodingPreset {
    /// Returns the ffmpeg command-line name for this preset.
    pub fn as_str(&self) -> &'static str {
        match self {
            EncodingPreset::Ultrafast => "ultrafast",
            EncodingPreset::Superfast => "superfast",
            EncodingPreset::Veryfast => "veryfast",
            EncodingPreset::Faster => "faster",
            EncodingPreset::Fast => "fast",
            EncodingPreset::Medium => "medium",
            EncodingPreset::Slow => "slow",
            EncodingPreset::Slower => "slower",
            EncodingPreset::Veryslow => "veryslow",
            EncodingPreset::Placebo => "placebo",
        }
    }

    /// Parses a preset name string, case-insensitive.
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "ultrafast" => Ok(EncodingPreset::Ultrafast),
            "superfast" => Ok(EncodingPreset::Superfast),
            "veryfast" => Ok(EncodingPreset::Veryfast),
            "faster" => Ok(EncodingPreset::Faster),
            "fast" => Ok(EncodingPreset::Fast),
            "medium" => Ok(EncodingPreset::Medium),
            "slow" => Ok(EncodingPreset::Slow),
            "slower" => Ok(EncodingPreset::Slower),
            "veryslow" => Ok(EncodingPreset::Veryslow),
            "placebo" => Ok(EncodingPreset::Placebo),
            _ => anyhow::bail!(
                "Invalid preset '{}'. Valid presets: ultrafast, superfast, veryfast, faster, fast, medium, slow, slower, veryslow, placebo",
                s
            ),
        }
    }
}

/// Encoding quality profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodingProfile {
    Quality,
    Balanced,
    Small,
}

impl EncodingProfile {
    /// Returns the default CRF and preset for this profile.
    pub fn defaults(self) -> (u32, EncodingPreset) {
        match self {
            EncodingProfile::Quality => (18, EncodingPreset::Medium),
            EncodingProfile::Balanced => (21, EncodingPreset::Veryfast),
            EncodingProfile::Small => (24, EncodingPreset::Veryfast),
        }
    }

    /// Parses a profile name string, case-insensitive.
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "quality" => Ok(EncodingProfile::Quality),
            "balanced" => Ok(EncodingProfile::Balanced),
            "small" => Ok(EncodingProfile::Small),
            _ => anyhow::bail!(
                "Invalid profile '{}'. Valid profiles: quality, balanced, small",
                s
            ),
        }
    }
}

/// Encoding configuration for H.264/MP4 output.
#[derive(Debug, Clone)]
pub struct EncodingConfig {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub crf: u32,
    pub preset: EncodingPreset,
}

impl EncodingConfig {
    /// Creates a new config from a profile, applying optional overrides.
    pub fn new(
        profile: EncodingProfile,
        crf: Option<u32>,
        preset: Option<EncodingPreset>,
        width: u32,
        height: u32,
        fps: u32,
    ) -> Result<Self> {
        let (default_crf, default_preset) = profile.defaults();
        let crf = crf.unwrap_or(default_crf);
        let preset = preset.unwrap_or(default_preset);

        // Validate CRF range (x264 valid range is 0-51)
        if crf > 51 {
            anyhow::bail!("CRF must be between 0 and 51, got {}", crf);
        }

        // Validate resolution dimensions are even (required for yuv420p)
        if width > 0 && width % 2 != 0 {
            anyhow::bail!("Width must be an even number for yuv420p output, got {}", width);
        }
        if height > 0 && height % 2 != 0 {
            anyhow::bail!("Height must be an even number for yuv420p output, got {}", height);
        }

        // Validate FPS is reasonable
        if fps == 0 || fps > 240 {
            anyhow::bail!("FPS must be between 1 and 240, got {}", fps);
        }

        Ok(Self {
            width,
            height,
            fps,
            crf,
            preset,
        })
    }
}

/// MP4 muxer for H.264 video.
///
/// Spawns an `ffmpeg` subprocess that consumes raw BGRA frames on stdin and
/// writes an H.264/MP4 file.
pub struct Muxer {
    child: Option<std::process::Child>,
    stdin: Option<std::process::ChildStdin>,
    config: EncodingConfig,
    frames_written: u64,
}

impl Muxer {
    /// Create a new MP4 muxer.
    pub fn new(output_path: &str, config: &EncodingConfig) -> Result<Self> {
        eprintln!(
            "Muxer: initializing MP4 output at {} ({}x{}@{}, CRF={}, preset={})",
            output_path, config.width, config.height, config.fps, config.crf,
            config.preset.as_str()
        );

        if let Some(parent) = Path::new(output_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut child = Command::new("ffmpeg")
            .args([
                "-y",
                "-f",
                "rawvideo",
                "-pix_fmt",
                "bgra",
                "-s",
                &format!("{}x{}", config.width, config.height),
                "-r",
                &config.fps.to_string(),
                "-i",
                "-",
                "-c:v",
                "libx264",
                "-preset",
                config.preset.as_str(),
                "-crf",
                &config.crf.to_string(),
                "-pix_fmt",
                "yuv420p",
                "-movflags",
                "+faststart",
                "-f",
                "mp4",
                output_path,
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("ffmpeg not found or failed to start (install ffmpeg and ensure it is on PATH)")?;

        let stdin = child
            .stdin
            .take()
            .context("ffmpeg has no stdin")?;

        Ok(Self {
            child: Some(child),
            stdin: Some(stdin),
            config: config.clone(),
            frames_written: 0,
        })
    }

    /// Write an encoded video packet to the output file.
    pub fn write_video(&mut self, packet: &crate::encode::EncodedPacket) -> Result<()> {
        let stdin = self.stdin.as_mut().context("muxer not initialized")?;
        stdin.write_all(&packet.data).context("ffmpeg write failed")?;
        self.frames_written += 1;
        Ok(())
    }

    /// Finalize the MP4 file and shut down the ffmpeg subprocess.
    pub fn finish(&mut self) -> Result<()> {
        if let Some(stdin) = self.stdin.take() {
            drop(stdin);
        }
        if let Some(mut child) = self.child.take() {
            child.wait().context("ffmpeg wait failed")?;
        }
        Ok(())
    }

    /// Get frames written count.
    pub fn frames_written(&self) -> u64 {
        self.frames_written
    }
}

impl Drop for Muxer {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}
