//! Performance metrics tracking.
//!
<<<<<<< HEAD
//! Tracks capture FPS, encode FPS, dropped frames, and timing information.

use std::time::Instant;

=======
//! Tracks capture FPS, encode FPS, dropped frames, timing information,
//! and CPU utilization. GPU utilization is not implemented because it
//! requires vendor-specific APIs (NVML, AMD ADL, Intel GMM) that are
//! not universally available.

use std::time::Instant;

/// A minimal FILETIME-compatible structure for GetSystemTimes FFI.
#[repr(C)]
struct WinFileTime {
    dw_low_date_time: u32,
    dw_high_date_time: u32,
}

extern "system" {
    fn GetSystemTimes(
        lp_idle_time: *mut WinFileTime,
        lp_kernel_time: *mut WinFileTime,
        lp_user_time: *mut WinFileTime,
    ) -> i32;
}

/// Convert a WinFileTime to nanoseconds since the FILETIME epoch.
fn win_filetime_to_ns(ft: &WinFileTime) -> u64 {
    let low = ft.dw_low_date_time as u64;
    let high = ft.dw_high_date_time as u64;
    let hundred_ns = (high << 32) | low;
    // FILETIME is in 100-nanosecond intervals since 1601-01-01.
    // Convert to nanoseconds.
    hundred_ns * 100
}

/// CPU time sample for utilization calculation.
#[derive(Debug, Clone, Copy)]
struct CpuSample {
    idle_ns: u64,
    kernel_ns: u64,
    user_ns: u64,
}

>>>>>>> b7baee3 (Milestone 6)
/// Frame timing metrics for the recording pipeline.
pub struct Metrics {
    start_time: Instant,
    frames_captured: u64,
    frames_encoded: u64,
    frames_dropped: u64,
    total_encode_time_ns: u64,
    last_stats_print: Instant,
<<<<<<< HEAD
=======
    cpu_samples: Vec<CpuSample>,
    last_cpu_sample: Option<CpuSample>,
    cpu_utilization: f64,
>>>>>>> b7baee3 (Milestone 6)
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
<<<<<<< HEAD
=======
            cpu_samples: Vec::new(),
            last_cpu_sample: None,
            cpu_utilization: 0.0,
>>>>>>> b7baee3 (Milestone 6)
        }
    }

    /// Record a captured frame.
    pub fn frame_captured(&mut self) {
        self.frames_captured += 1;
<<<<<<< HEAD
=======
        self.update_cpu_utilization();
>>>>>>> b7baee3 (Milestone 6)
    }

    /// Record an encoded frame with encoding time.
    pub fn frame_encoded(&mut self, encode_time_ns: u64) {
        self.frames_encoded += 1;
        self.total_encode_time_ns += encode_time_ns;
<<<<<<< HEAD
=======
        self.update_cpu_utilization();
>>>>>>> b7baee3 (Milestone 6)
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

<<<<<<< HEAD
=======
    /// Get current CPU utilization percentage (0.0 to 100.0).
    pub fn cpu_utilization(&self) -> f64 {
        self.cpu_utilization
    }

    /// Get GPU utilization percentage. Not implemented - requires vendor-specific APIs.
    /// Returns `None` to indicate unavailable.
    pub fn gpu_utilization(&self) -> Option<f64> {
        None
    }

>>>>>>> b7baee3 (Milestone 6)
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
<<<<<<< HEAD
            "\rCaptured: {} | Encoded: {} | CapFPS: {:.1} | EncFPS: {:.1} | AvgEnc: {:.2}ms | Dropped: {}",
=======
            "\rCaptured: {} | Encoded: {} | CapFPS: {:.1} | EncFPS: {:.1} | AvgEnc: {:.2}ms | CPU: {:.1}% | Dropped: {}",
>>>>>>> b7baee3 (Milestone 6)
            self.frames_captured,
            self.frames_encoded,
            self.capture_fps(),
            self.encode_fps(),
            self.avg_encode_ms(),
<<<<<<< HEAD
            self.frames_dropped
        );
    }
=======
            self.cpu_utilization,
            self.frames_dropped
        );
    }

    /// Update CPU utilization using GetSystemTimes.
    fn update_cpu_utilization(&mut self) {
        let mut idle: WinFileTime = unsafe { std::mem::zeroed() };
        let mut kernel: WinFileTime = unsafe { std::mem::zeroed() };
        let mut user: WinFileTime = unsafe { std::mem::zeroed() };

        unsafe {
            if GetSystemTimes(
                &mut idle as *mut WinFileTime,
                &mut kernel as *mut WinFileTime,
                &mut user as *mut WinFileTime,
            ) != 0
            {
                let idle_ns = win_filetime_to_ns(&idle);
                let kernel_ns = win_filetime_to_ns(&kernel);
                let user_ns = win_filetime_to_ns(&user);

                let sample = CpuSample {
                    idle_ns,
                    kernel_ns,
                    user_ns,
                };

                if let Some(prev) = self.last_cpu_sample {
                    let idle_delta = sample.idle_ns.saturating_sub(prev.idle_ns);
                    let total_delta = (sample.kernel_ns - prev.kernel_ns)
                        .saturating_add(sample.user_ns - prev.user_ns);

                    if total_delta > 0 {
                        let utilization =
                            100.0 * (total_delta as f64 - idle_delta as f64) / total_delta as f64;
                        self.cpu_utilization = utilization.clamp(0.0, 100.0);
                    }
                }

                self.last_cpu_sample = Some(sample);
                self.cpu_samples.push(sample);

                // Keep only the last 60 samples (~1 minute at 1 sample/sec)
                if self.cpu_samples.len() > 60 {
                    self.cpu_samples.remove(0);
                }
            }
        }
    }
>>>>>>> b7baee3 (Milestone 6)
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
<<<<<<< HEAD
}
=======
}
>>>>>>> b7baee3 (Milestone 6)
