//! Screen capture using Windows Graphics Capture (WGC) via the `wgc` crate.
//!
//! Provides WGC session initialization and frame iteration.
//! Frames are returned as BGRA byte buffers with timestamps and dimensions.

use anyhow::Result;
use windows::Win32::Foundation::{HWND, LPARAM, POINT};
use windows::Win32::Graphics::Gdi::{MonitorFromPoint, MONITOR_DEFAULTTOPRIMARY};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
};
use wgc::{PixelFormat, Wgc};

/// Represents a single captured frame.
pub struct Frame {
    /// Width of the frame in pixels.
    pub width: u32,
    /// Height of the frame in pixels.
    pub height: u32,
    /// BGRA pixel data (4 bytes per pixel, stride = width * 4).
    pub pixels: Vec<u8>,
    /// Monotonic timestamp in nanoseconds.
    pub timestamp_ns: u64,
}

/// Information about an enumerable top-level window.
#[derive(Debug, Clone)]
pub struct WindowInfo {
    /// Window handle.
    pub hwnd: HWND,
    /// Window title text.
    pub title: String,
    /// Process ID owning the window.
    pub process_id: u32,
}

/// Enumerate all visible top-level windows.
pub fn enumerate_windows() -> Result<Vec<WindowInfo>> {
    let mut windows: Vec<WindowInfo> = Vec::new();

    unsafe {
        EnumWindows(
            Some(enum_windows_callback),
            LPARAM(&mut windows as *mut Vec<WindowInfo> as isize),
        )?;
    }

    Ok(windows)
}

unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> windows::core::BOOL {
    if unsafe { !IsWindowVisible(hwnd).as_bool() } {
        return windows::Win32::Foundation::TRUE;
    }

    let mut title_buf = [0u16; 512];
    let len = unsafe { GetWindowTextW(hwnd, &mut title_buf) };
    if len == 0 {
        return windows::Win32::Foundation::TRUE;
    }

    let title: String = title_buf[..len as usize]
        .iter()
        .map(|&c| char::from_u32(c as u32).unwrap_or('\u{FFFD}'))
        .collect();

    let mut process_id: u32 = 0;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut process_id)) };

    let windows: &mut Vec<WindowInfo> =
        unsafe { &mut *(lparam.0 as *mut Vec<WindowInfo>) };
    windows.push(WindowInfo {
        hwnd,
        title,
        process_id,
    });

    windows::Win32::Foundation::TRUE
}

/// Find a window by matching its title (case-insensitive substring match).
/// Returns the first matching window.
pub fn find_window_by_title(title_pattern: &str) -> Result<HWND> {
    let windows = enumerate_windows()?;
    let pattern_lower = title_pattern.to_lowercase();

    for w in &windows {
        if w.title.to_lowercase().contains(&pattern_lower) {
            return Ok(w.hwnd);
        }
    }

    anyhow::bail!("No visible window found with title matching '{}'", title_pattern)
}

/// Screen capture using Windows Graphics Capture.
pub struct Capture {
    wgc: Option<Wgc>,
    /// The first captured frame, held until the caller consumes it. Used to
    //  determine capture dimensions before any frame has been pulled.
    pending: Option<Frame>,
    width: u32,
    height: u32,
    frame_count: u64,
    start_time_ns: u64,
}

impl Capture {
    /// Create a new capture session from a window handle.
    pub fn from_window(hwnd: HWND) -> Result<Self> {
        let wgc = Wgc::new_from_hwnd(hwnd)?
            .pixel_format(PixelFormat::BGRA8)
            .frame_queue_length(3)
            .capture_cursor(false)
            .build()?;
        let mut cap = Self {
            wgc: Some(wgc),
            pending: None,
            width: 0,
            height: 0,
            frame_count: 0,
            start_time_ns: now_ns(),
        };
        // Prime dimensions by pulling the first frame (blocks until available).
        cap.pending = cap.pull_frame()?;
        if let Some(ref frame) = cap.pending {
            cap.width = frame.width;
            cap.height = frame.height;
        }
        Ok(cap)
    }

    /// Create a new capture session from the primary monitor (desktop capture).
    pub fn from_monitor() -> Result<Self> {
        // Get the primary monitor handle
        let primary_hmonitor = unsafe {
            MonitorFromPoint(
                POINT { x: 0, y: 0 },
                MONITOR_DEFAULTTOPRIMARY,
            )
        };

        let wgc = Wgc::new_from_monitor(primary_hmonitor)?
            .pixel_format(PixelFormat::BGRA8)
            .frame_queue_length(3)
            .capture_cursor(false)
            .build()?;
        let mut cap = Self {
            wgc: Some(wgc),
            pending: None,
            width: 0,
            height: 0,
            frame_count: 0,
            start_time_ns: now_ns(),
        };
        cap.pending = cap.pull_frame()?;
        if let Some(ref frame) = cap.pending {
            cap.width = frame.width;
            cap.height = frame.height;
        }
        Ok(cap)
    }

    /// Get the capture width.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the capture height.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get total frames captured.
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Get average capture FPS since start.
    pub fn fps(&self) -> f64 {
        let elapsed = (now_ns() - self.start_time_ns) as f64 / 1_000_000_000.0;
        if elapsed > 0.0 {
            self.frame_count as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Capture the next available frame.
    /// Blocks until a frame is available from the WGC session.
    pub fn next_frame(&mut self) -> Result<Option<Frame>> {
        // Return the primed first frame before pulling from the iterator.
        if let Some(frame) = self.pending.take() {
            self.frame_count += 1;
            return Ok(Some(frame));
        }
        let frame = self.pull_frame()?;
        if let Some(ref f) = frame {
            self.frame_count += 1;
            self.width = f.width;
            self.height = f.height;
        }
        Ok(frame)
    }

    /// Pull a single frame from the WGC iterator.
    fn pull_frame(&mut self) -> Result<Option<Frame>> {
        let Some(wgc) = self.wgc.as_mut() else {
            return Ok(None);
        };
        match wgc.next() {
            Some(Ok(wgc_frame)) => {
                let pixels = wgc_frame.pixels()?;
                let size = wgc_frame.size()?;
                Ok(Some(Frame {
                    width: size.width,
                    height: size.height,
                    pixels,
                    timestamp_ns: now_ns(),
                }))
            }
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    /// Destroy the capture session.
    pub fn stop(&mut self) -> Result<()> {
        self.wgc = None;
        self.pending = None;
        Ok(())
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

/// Get current time in nanoseconds using the Windows performance counter.
fn now_ns() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_now_ns_returns_nonzero() {
        let t = now_ns();
        assert!(t > 0);
    }
}
