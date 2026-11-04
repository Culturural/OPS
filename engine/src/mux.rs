//! MP4 muxing module.
//!
//! Muxes captured video into an MP4 container by piping raw BGRA frames to an
//! `ffmpeg` subprocess, which performs H.264 encoding and MP4 muxing,
//! producing a playable MP4 file.
//!
//! Supported encoders:
//!   - libx264  (software, CRF-based)
//!   - h264_nvenc (NVIDIA NVENC, CQP/CQ-based)
//!   - h264_amf   (AMD AMF, QP-based)
//!   - h264_qsv   (Intel Quick Sync, QP/ICQ-based)

use anyhow::{Context, Result};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

/// Encoder selection for H.264/MP4 output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncoderType {
    /// Software x264 encoding (always available if ffmpeg is installed).
    Libx264,
    /// NVIDIA NVENC hardware encoding.
    Nvenc,
    /// AMD AMF hardware encoding.
    Amf,
    /// Intel Quick Sync hardware encoding.
    Qsv,
}

impl EncoderType {
    /// Returns the ffmpeg codec name for this encoder.
    pub fn codec_name(&self) -> &'static str {
        match self {
            EncoderType::Libx264 => "libx264",
            EncoderType::Nvenc => "h264_nvenc",
            EncoderType::Amf => "h264_amf",
            EncoderType::Qsv => "h264_qsv",
        }
    }

    /// Returns a human-readable name.
    pub fn display_name(&self) -> &'static str {
        match self {
            EncoderType::Libx264 => "libx264 (software)",
            EncoderType::Nvenc => "h264_nvenc (NVIDIA NVENC)",
            EncoderType::Amf => "h264_amf (AMD AMF)",
            EncoderType::Qsv => "h264_qsv (Intel Quick Sync)",
        }
    }

    /// Parses an encoder name string, case-insensitive.
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "auto" | "libx264" | "x264" => Ok(EncoderType::Libx264),
            "nvenc" => Ok(EncoderType::Nvenc),
            "amf" => Ok(EncoderType::Amf),
            "qsv" => Ok(EncoderType::Qsv),
            _ => anyhow::bail!(
                "Invalid encoder '{}'. Valid encoders: auto, libx264, nvenc, amf, qsv",
                s
            ),
        }
    }
}

/// Encoding preset for software x264.
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
    /// Returns the default CRF/CQP and preset for this profile when using libx264.
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
///
/// CRF is specific to x264-style rate control. Hardware encoders (NVENC, AMF, QSV)
/// use different quality controls (CQP, ICQ, vendor-specific settings) and may not
/// accept CRF at all. Do not apply x264 CRF assumptions to hardware encoders.
#[derive(Debug, Clone)]
pub struct EncodingConfig {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub encoder_type: EncoderType,
    /// CRF value for libx264 (0-51). Ignored by hardware encoders.
    pub crf: u32,
    /// Quality level for hardware encoders (encoder-specific scale).
    /// Ignored by libx264.
    pub quality_level: u32,
    pub preset: EncodingPreset,
}

impl EncodingConfig {
    /// Creates a new config from a profile, applying optional overrides.
    pub fn new(
        profile: EncodingProfile,
        encoder_type: EncoderType,
        crf: Option<u32>,
        quality_level: Option<u32>,
        preset: Option<EncodingPreset>,
        width: u32,
        height: u32,
        fps: u32,
    ) -> Result<Self> {
        let (default_crf, default_preset) = profile.defaults();
        let crf = crf.unwrap_or(default_crf);
        let preset = preset.unwrap_or(default_preset);

        // Validate CRF range (x264 valid range is 0-51) — only applies to libx264.
        if crf > 51 {
            anyhow::bail!("CRF must be between 0 and 51, got {}", crf);
        }

        // Validate quality level for hardware encoders (typical range 0-51).
        let quality_level = quality_level.unwrap_or(crf);
        if quality_level > 51 {
            anyhow::bail!(
                "Quality level must be between 0 and 51, got {}",
                quality_level
            );
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
            encoder_type,
            crf,
            quality_level,
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
    encoder_type: EncoderType,
}

impl Muxer {
    /// Create a new MP4 muxer.
    pub fn new(output_path: &str, config: &EncodingConfig) -> Result<Self> {
        let encoder_type = config.encoder_type;

        eprintln!(
            "Muxer: initializing MP4 output at {} ({}x{}@{}, encoder={})",
            output_path, config.width, config.height, config.fps,
            encoder_type.display_name()
        );

        if let Some(parent) = Path::new(output_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut args: Vec<String> = Vec::new();
        args.push("-y".to_string());
        args.push("-f".to_string());
        args.push("rawvideo".to_string());
        args.push("-pix_fmt".to_string());
        args.push("bgra".to_string());
        args.push("-s".to_string());
        args.push(format!("{}x{}", config.width, config.height));
        args.push("-r".to_string());
        args.push(config.fps.to_string());
        args.push("-i".to_string());
        args.push("-".to_string());

        // Encoder-specific options.
        match encoder_type {
            EncoderType::Libx264 => {
                args.push("-c:v".to_string());
                args.push("libx264".to_string());
                args.push("-preset".to_string());
                args.push(config.preset.as_str().to_string());
                args.push("-crf".to_string());
                args.push(config.crf.to_string());
            }
            EncoderType::Nvenc => {
                args.push("-c:v".to_string());
                args.push("h264_nvenc".to_string());
                // NVENC uses -preset (slow/medium/fast/hp/hq/ll/llhq/llhp/lossless)
                // and -cq for constant quality (CQP mode).
                args.push("-preset".to_string());
                args.push(config.preset.as_str().to_string());
                args.push("-cq".to_string());
                args.push(config.quality_level.to_string());
            }
            EncoderType::Amf => {
                args.push("-c:v".to_string());
                args.push("h264_amf".to_string());
                // AMF uses -quality (balanced/speed) and -qp (quantization parameter).
                args.push("-quality".to_string());
                args.push(match config.preset {
                    EncodingPreset::Ultrafast | EncodingPreset::Superfast | EncodingPreset::Veryfast | EncodingPreset::Faster => "speed",
                    _ => "balanced",
                }.to_string());
                args.push("-qp".to_string());
                args.push(config.quality_level.to_string());
            }
            EncoderType::Qsv => {
                args.push("-c:v".to_string());
                args.push("h264_qsv".to_string());
                // QSV uses -look_ahead 0 (disable lookahead for low latency) and -qp.
                args.push("-look_ahead".to_string());
                args.push("0".to_string());
                args.push("-qp".to_string());
                args.push(config.quality_level.to_string());
            }
        }

        args.push("-pix_fmt".to_string());
        args.push("yuv420p".to_string());
        args.push("-movflags".to_string());
        args.push("+faststart".to_string());
        args.push("-f".to_string());
        args.push("mp4".to_string());
        args.push(output_path.to_string());

        let mut child = Command::new("ffmpeg")
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context(format!(
                "ffmpeg not found or failed to start (install ffmpeg and ensure it is on PATH). Encoder: {}",
                encoder_type.codec_name()
            ))?;

        let stdin = child
            .stdin
            .take()
            .context("ffmpeg has no stdin")?;

        Ok(Self {
            child: Some(child),
            stdin: Some(stdin),
            config: config.clone(),
            frames_written: 0,
            encoder_type,
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

    /// Get the encoder type in use.
    pub fn encoder_type(&self) -> EncoderType {
        self.encoder_type
    }
}

impl Drop for Muxer {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}
