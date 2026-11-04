//! Video encoding module.
//!
//! Wraps Windows Media Foundation H.264 hardware encoder (MFT) with software fallback.
//!
//! Pipeline: BGRA frames → [optional BGRA→NV12 convert] → Hardware Encoder MFT → H.264 bitstream packets.

use anyhow::Result;
use std::time::Instant;

<<<<<<< HEAD
<<<<<<< HEAD
=======
use crate::mux::EncoderType;

>>>>>>> b7baee3 (Milestone 6)
=======
>>>>>>> origin/master
/// Encoded video packet ready for muxing.
pub struct EncodedPacket {
    /// H.264 NAL unit bitstream data.
    pub data: Vec<u8>,
    /// Presentation timestamp in nanoseconds.
    pub timestamp_ns: u64,
    /// Is this a keyframe (IDR)?
    pub is_keyframe: bool,
}

<<<<<<< HEAD
<<<<<<< HEAD
=======
>>>>>>> origin/master
/// Hardware encoder selection result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncoderType {
    /// NVIDIA NVENC
    Nvenc,
    /// AMD AMF
    Amf,
    /// Intel Quick Sync
    QuickSync,
    /// Software fallback (x264/SVT-AV1)
    Software,
}

/// Video encoder wrapper.
<<<<<<< HEAD
=======
/// Video encoder wrapper.
///
/// Note: Actual H.264 encoding is performed by ffmpeg in the muxer subprocess.
/// This module packages captured frames into packets for the muxer and tracks
/// encoder selection for reporting purposes.
>>>>>>> b7baee3 (Milestone 6)
=======
>>>>>>> origin/master
pub struct Encoder {
    encoder_type: EncoderType,
    width: u32,
    height: u32,
    fps: u32,
    frame_count: u64,
    start_time: Instant,
    encoded_count: u64,
}

impl Encoder {
<<<<<<< HEAD
<<<<<<< HEAD
=======
>>>>>>> origin/master
    /// Create a new encoder, detecting and selecting the best available hardware encoder.
    pub fn new(width: u32, height: u32, fps: u32) -> Result<Self> {
        let encoder_type = Self::detect_best_encoder()?;
        eprintln!(
            "Selected encoder: {:?}",
            encoder_type
        );

<<<<<<< HEAD
=======
    /// Create a new encoder with the specified type.
    pub fn new(width: u32, height: u32, fps: u32, encoder_type: EncoderType) -> Result<Self> {
>>>>>>> b7baee3 (Milestone 6)
=======
>>>>>>> origin/master
        Ok(Self {
            encoder_type,
            width,
            height,
            fps,
            frame_count: 0,
            start_time: Instant::now(),
            encoded_count: 0,
        })
    }

<<<<<<< HEAD
<<<<<<< HEAD
=======
>>>>>>> origin/master
    /// Detect the best available hardware encoder.
    fn detect_best_encoder() -> Result<EncoderType> {
        // TODO: Implement actual hardware detection via DirectX adapter enumeration
        // For now, default to software fallback which will be implemented with Media Foundation
        Ok(EncoderType::Software)
<<<<<<< HEAD
=======
    /// Get encoder type.
    pub fn encoder_type(&self) -> EncoderType {
        self.encoder_type
>>>>>>> b7baee3 (Milestone 6)
=======
>>>>>>> origin/master
    }

    /// Get encoder type name for display.
    pub fn name(&self) -> String {
<<<<<<< HEAD
<<<<<<< HEAD
=======
>>>>>>> origin/master
        match self.encoder_type {
            EncoderType::Nvenc => "NVENC (NVIDIA)".to_string(),
            EncoderType::Amf => "AMF (AMD)".to_string(),
            EncoderType::QuickSync => "QuickSync (Intel)".to_string(),
            EncoderType::Software => "Software (CPU)".to_string(),
        }
<<<<<<< HEAD
=======
        self.encoder_type.display_name().to_string()
>>>>>>> b7baee3 (Milestone 6)
=======
>>>>>>> origin/master
    }

    /// Encode a captured frame into H.264 packets.
    /// Returns `Ok(Some(packet))` when a packet is ready, `Ok(None)` when more input needed.
    pub fn encode_frame(&mut self, frame: &crate::capture::Frame) -> Result<Option<EncodedPacket>> {
        self.frame_count += 1;

<<<<<<< HEAD
<<<<<<< HEAD
        // The Media Foundation sink writer (owned by the muxer) performs the real
=======
        // The ffmpeg subprocess (owned by the muxer) performs the real
>>>>>>> b7baee3 (Milestone 6)
=======
        // The Media Foundation sink writer (owned by the muxer) performs the real
>>>>>>> origin/master
        // H.264 encoding. This stage packages the captured BGRA frame and its
        // timestamp as a packet for the muxer to encode and write.
        Ok(Some(EncodedPacket {
            data: frame.pixels.clone(),
            timestamp_ns: frame.timestamp_ns,
            is_keyframe: false,
        }))
    }

    /// Flush encoder and return remaining packets.
    pub fn finish(&mut self) -> Result<Vec<EncodedPacket>> {
        Ok(Vec::new())
    }

    /// Get encoded frames per second.
    pub fn fps(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.encoded_count as f64 / elapsed
        } else {
            0.0
        }
    }
}

impl Default for Encoder {
    fn default() -> Self {
<<<<<<< HEAD
<<<<<<< HEAD
        Self::new(1280, 720, 30).unwrap()
=======
        Self::new(1280, 720, 30, EncoderType::Libx264).unwrap()
>>>>>>> b7baee3 (Milestone 6)
=======
        Self::new(1280, 720, 30).unwrap()
>>>>>>> origin/master
    }
}