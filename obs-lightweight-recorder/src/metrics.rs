//! Performance metrics tracking.
//!
//! Tracks capture FPS, encode FPS, dropped frames, and timing information.

use std::time::Instant;

/// Frame timing metrics for the recording pipeline.
pub struct Metrics {
    start_time: Instant,
    frames_captured: u64,
    frames_encoded: u64,
    frames_dropped: u64,
    total_encode_time_ns: u64,
    last_stats_print: Instant,
}

impl Metrics {
    /// Create a new metrics tracker.
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            frames_captured: 0,
            frames_encoded: 0,
            frames_dropped: 0,
            total_encode_time_ns: 0,
            last_stats_print: Instant::now(),
        }
    }

    /// Record a captured frame.
    pub fn frame_captured(&mut self) {
        self.frames_captured += 1;
    }

    /// Record an encoded frame with encoding time.
    pub fn frame_encoded(&mut self, encode_time_ns: u64) {
        self.frames_encoded += 1;
        self.total_encode_time_ns += encode_time_ns;
    }

    /// Record a dropped frame.
    pub fn frame_dropped(&mut self) {
        self.frames_dropped += 1;
    }

    /// Get capture FPS.
    pub fn capture_fps(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.frames_captured as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Get encode FPS.
    pub fn encode_fps(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.frames_encoded as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Get average encode time per frame in milliseconds.
    pub fn avg_encode_ms(&self) -> f64 {
        if self.frames_encoded > 0 {
            self.total_encode_time_ns as f64 / self.frames_encoded as f64 / 1_000_000.0
        } else {
            0.0
        }
    }

    /// Get dropped frames count.
    pub fn dropped(&self) -> u64 {
        self.frames_dropped
    }

    /// Get total captured frames.
    pub fn captured(&self) -> u64 {
        self.frames_captured
    }

    /// Get total encoded frames.
    pub fn encoded(&self) -> u64 {
        self.frames_encoded
    }

    /// Check if it's time to print stats (every ~1 second).
    pub fn should_print_stats(&self) -> bool {
        self.last_stats_print.elapsed() >= std::time::Duration::from_secs(1)
    }

    /// Mark stats as printed.
    pub fn mark_stats_printed(&mut self) {
        self.last_stats_print = Instant::now();
    }

    /// Print current stats to stderr.
    pub fn print_stats(&self) {
        eprint!(
            "\rCaptured: {} | Encoded: {} | CapFPS: {:.1} | EncFPS: {:.1} | AvgEnc: {:.2}ms | Dropped: {}",
            self.frames_captured,
            self.frames_encoded,
            self.capture_fps(),
            self.encode_fps(),
            self.avg_encode_ms(),
            self.frames_dropped
        );
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}