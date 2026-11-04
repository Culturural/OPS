//! Local hardware detection for adaptive recording configuration.
//!
//! Inspects CPU, RAM, GPU, Windows version, and display resolution using
//! native Windows APIs. All detection is local; no data is transmitted.

use anyhow::{Context, Result};
use std::mem;
use std::ptr;
use windows::core::PCWSTR;
use windows::Win32::System::Com::{CoInitializeEx, CoTaskMemFree, CoUninitialize, COINIT_APARTMENTTHREADED};
use windows::Win32::System::Registry::{RegGetValueW, HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ};
use windows::Win32::System::SystemInformation::{
    GlobalMemoryStatusEx, GetSystemInfo, GetVersionExW, MEMORYSTATUSEX, OSVERSIONINFOW, SYSTEM_INFO,
};
use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory1, IDXGIAdapter1, IDXGIFactory1,
};
use windows::Win32::Graphics::Gdi::{
    CreateDCW, DeleteDC, GetDeviceCaps, HDC, HORZRES, VERTRES,
};
use windows::Win32::Media::MediaFoundation::{
    MFStartup, MFShutdown, MF_VERSION, MFTEnumEx, MFT_ENUM_FLAG,
    MFT_CATEGORY_VIDEO_ENCODER, MFT_FRIENDLY_NAME_Attribute,
};

/// Complete hardware profile detected from the local system.
#[derive(Debug, Clone)]
pub struct HardwareInfo {
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

/// Recommended encoding profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecommendedProfile {
    Performance,
    Balanced,
    Quality,
}

impl RecommendedProfile {
    pub fn as_str(&self) -> &'static str {
        match self {
            RecommendedProfile::Performance => "performance",
            RecommendedProfile::Balanced => "balanced",
            RecommendedProfile::Quality => "quality",
        }
    }
}

/// Recommended automatic configuration.
#[derive(Debug, Clone)]
pub struct AutoConfig {
    pub profile: RecommendedProfile,
    pub encoder: String,
    pub crf: u32,
    pub preset: String,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub reason: String,
}

impl HardwareInfo {
    /// Detect all available hardware information.
    pub fn detect() -> Result<Self> {
        let cpu = Self::detect_cpu()?;
        let ram = Self::detect_ram()?;
        let gpu = Self::detect_gpu()?;
        let windows = Self::detect_windows()?;
        let display = Self::detect_display()?;
        let encoders = Self::detect_encoders()?;

        Ok(HardwareInfo {
            cpu_manufacturer: cpu.0,
            cpu_model: cpu.1,
            cpu_cores: cpu.2,
            cpu_threads: cpu.3,
            total_ram_mb: ram,
            gpu_manufacturer: gpu.0,
            gpu_model: gpu.1,
            gpu_adapter_count: gpu.2,
            windows_version: windows.0,
            windows_build: windows.1,
            display_width: display.0,
            display_height: display.1,
            hardware_encoder_available: encoders.0,
            nvenc_available: encoders.1,
            amf_available: encoders.2,
            quicksync_available: encoders.3,
        })
    }

    /// Print detected hardware summary to stderr.
    pub fn print_summary(&self) {
        eprintln!("=== Detected Hardware ===");
        eprintln!("CPU: {} {}", self.cpu_manufacturer, self.cpu_model);
        eprintln!("  Cores: {} | Threads: {}", self.cpu_cores, self.cpu_threads);
        eprintln!("  RAM: {} MB", self.total_ram_mb);
        if let (Some(ref vendor), Some(ref model)) = (&self.gpu_manufacturer, &self.gpu_model) {
            eprintln!("GPU: {} {}", vendor, model);
        } else {
            eprintln!("GPU: Unknown");
        }
        eprintln!("GPU adapters: {}", self.gpu_adapter_count);
        eprintln!(
            "Windows: {} (build {})",
            self.windows_version, self.windows_build
        );
        eprintln!(
            "Display: {}x{}",
            self.display_width, self.display_height
        );
        eprintln!(
            "Hardware encoders: NVENC={} AMF={} QuickSync={}",
            self.nvenc_available, self.amf_available, self.quicksync_available
        );
        eprintln!("========================");
    }

    /// Recommend an encoding profile and configuration based on detected hardware.
    pub fn recommend_config(&self) -> AutoConfig {
        let has_nvenc = self.nvenc_available;
        let has_amf = self.amf_available;
        let has_quicksync = self.quicksync_available;
        let high_end_cpu = self.cpu_threads >= 8 && self.total_ram_mb >= 16384;
        let low_end_cpu = self.cpu_threads <= 4 || self.total_ram_mb <= 8192;

        if has_nvenc {
            AutoConfig {
                profile: RecommendedProfile::Performance,
                encoder: "nvenc".to_string(),
                crf: 23,
                preset: "veryfast".to_string(),
                width: self.display_width,
                height: self.display_height,
                fps: 30,
                reason: "NVIDIA NVENC hardware encoder detected (using libx264 software encoder)".to_string(),
            }
        } else if has_amf {
            AutoConfig {
                profile: RecommendedProfile::Performance,
                encoder: "amf".to_string(),
                crf: 23,
                preset: "veryfast".to_string(),
                width: self.display_width,
                height: self.display_height,
                fps: 30,
                reason: "AMD AMF hardware encoder detected (using libx264 software encoder)".to_string(),
            }
        } else if has_quicksync {
            AutoConfig {
                profile: RecommendedProfile::Balanced,
                encoder: "qsv".to_string(),
                crf: 23,
                preset: "veryfast".to_string(),
                width: self.display_width,
                height: self.display_height,
                fps: 30,
                reason: "Intel Quick Sync hardware encoder detected (using libx264 software encoder)".to_string(),
            }
        } else if high_end_cpu {
            AutoConfig {
                profile: RecommendedProfile::Balanced,
                encoder: "libx264".to_string(),
                crf: 21,
                preset: "veryfast".to_string(),
                width: self.display_width,
                height: self.display_height,
                fps: 30,
                reason: "High-end CPU detected; software x264 with balanced settings".to_string(),
            }
        } else if low_end_cpu {
            AutoConfig {
                profile: RecommendedProfile::Performance,
                encoder: "libx264".to_string(),
                crf: 24,
                preset: "veryfast".to_string(),
                width: self.display_width,
                height: self.display_height,
                fps: 30,
                reason: "Low-end CPU/RAM detected; software x264 with performance settings".to_string(),
            }
        } else {
            AutoConfig {
                profile: RecommendedProfile::Balanced,
                encoder: "libx264".to_string(),
                crf: 21,
                preset: "veryfast".to_string(),
                width: self.display_width,
                height: self.display_height,
                fps: 30,
                reason: "Default balanced software x264 configuration".to_string(),
            }
        }
    }

    // ── CPU detection ──────────────────────────────────────────────

    fn detect_cpu() -> Result<(String, String, u32, u32)> {
        let mut sys_info: SYSTEM_INFO = unsafe { mem::zeroed() };
        unsafe { GetSystemInfo(&mut sys_info) };

        let cores = sys_info.dwNumberOfProcessors.max(1);
        let threads = cores;

        let model = Self::read_registry_cpu_model().unwrap_or_else(|_| "Unknown".to_string());

        let manufacturer = if model.to_lowercase().contains("intel") {
            "Intel".to_string()
        } else if model.to_lowercase().contains("amd") {
            "AMD".to_string()
        } else {
            "Unknown".to_string()
        };

        Ok((manufacturer, model, cores, threads))
    }

    fn read_registry_cpu_model() -> Result<String> {
        let key_name = "HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0";
        let value_name = "ProcessorNameString";

        let mut buffer = [0u16; 256];
        let mut buffer_size: u32 = (buffer.len() * 2) as u32;

        let wide_key: Vec<u16> = key_name.encode_utf16().chain(Some(0)).collect();
        let wide_value: Vec<u16> = value_name.encode_utf16().chain(Some(0)).collect();

        let status = unsafe {
            RegGetValueW(
                HKEY_LOCAL_MACHINE,
                PCWSTR::from_raw(wide_key.as_ptr()),
                PCWSTR::from_raw(wide_value.as_ptr()),
                RRF_RT_REG_SZ,
                None,
                Some(buffer.as_mut_ptr() as *mut _),
                Some(&mut buffer_size),
            )
        };

        if status == windows::Win32::Foundation::WIN32_ERROR(0) {
            let len = (buffer_size as usize) / 2;
            let s: String = buffer[..len]
                .iter()
                .take_while(|&&c| c != 0)
                .map(|&c| char::from_u32(c as u32).unwrap_or('\u{FFFD}'))
                .collect();
            Ok(s.trim().to_string())
        } else {
            anyhow::bail!("RegGetValueW failed with status {}", status.0)
        }
    }

    // ── RAM detection ──────────────────────────────────────────────

    fn detect_ram() -> Result<u64> {
        let mut mem_status: MEMORYSTATUSEX = unsafe { mem::zeroed() };
        mem_status.dwLength = mem::size_of::<MEMORYSTATUSEX>() as u32;

        unsafe { GlobalMemoryStatusEx(&mut mem_status) }?;

        Ok(mem_status.ullTotalPhys / (1024 * 1024))
    }

    // ── GPU detection ──────────────────────────────────────────────

    fn detect_gpu() -> Result<(Option<String>, Option<String>, u32)> {
        Ok(Self::detect_gpu_dxgi().unwrap_or_else(|_| (None, None, 0)))
    }

    fn detect_gpu_dxgi() -> Result<(Option<String>, Option<String>, u32)> {
        unsafe {
            let _ = CoInitializeEx(Some(ptr::null_mut()), COINIT_APARTMENTTHREADED);
        }

        let factory: IDXGIFactory1 = match unsafe { CreateDXGIFactory1() } {
            Ok(f) => f,
            Err(_) => {
                unsafe { CoUninitialize(); }
                return Ok((None, None, 0));
            }
        };

        let mut adapters: Vec<IDXGIAdapter1> = Vec::new();
        let mut index: u32 = 0;

        loop {
            match unsafe { factory.EnumAdapters1(index) } {
                Ok(adapter) => {
                    adapters.push(adapter);
                    index += 1;
                }
                Err(_) => break,
            }
        }

        let count = adapters.len() as u32;

        let (vendor, model) = if let Some(adapter) = adapters.first() {
            match unsafe { adapter.GetDesc1() } {
                Ok(desc) => {
                    let desc_str = String::from_utf16_lossy(&desc.Description);
                    let vendor = if desc_str.to_lowercase().contains("nvidia") {
                        Some("NVIDIA".to_string())
                    } else if desc_str.to_lowercase().contains("amd")
                        || desc_str.to_lowercase().contains("ati")
                    {
                        Some("AMD".to_string())
                    } else if desc_str.to_lowercase().contains("intel") {
                        Some("Intel".to_string())
                    } else {
                        None
                    };
                    (vendor, Some(desc_str.trim().to_string()))
                }
                Err(_) => (None, None),
            }
        } else {
            (None, None)
        };

        unsafe { CoUninitialize(); }

        Ok((vendor, model, count))
    }

    // ── Windows version detection ─────────────────────────────────

    fn detect_windows() -> Result<(String, u32)> {
        let mut os_info: OSVERSIONINFOW = unsafe { mem::zeroed() };
        os_info.dwOSVersionInfoSize = mem::size_of::<OSVERSIONINFOW>() as u32;

        match unsafe { GetVersionExW(&mut os_info) } {
            Ok(()) => {
                let version = format!(
                    "{}.{}.{}",
                    os_info.dwMajorVersion, os_info.dwMinorVersion, os_info.dwBuildNumber
                );
                Ok((version, os_info.dwBuildNumber))
            }
            Err(_) => Ok(("Windows".to_string(), 0)),
        }
    }

    // ── Display detection ──────────────────────────────────────────

    fn detect_display() -> Result<(u32, u32)> {
        let mut width = 0u32;
        let mut height = 0u32;

        unsafe {
            let hdc: HDC = CreateDCW(
                PCWSTR::from_raw("DISPLAY\0".encode_utf16().collect::<Vec<_>>().as_ptr()),
                None,
                None,
                None,
            );
            if !hdc.0.is_null() {
                width = GetDeviceCaps(Some(hdc), HORZRES) as u32;
                height = GetDeviceCaps(Some(hdc), VERTRES) as u32;
                let _ = DeleteDC(hdc);
            }
        }

        if width == 0 || height == 0 {
            width = 1920;
            height = 1080;
        }

        Ok((width, height))
    }

    // ── Encoder detection ──────────────────────────────────────────

    fn detect_encoders() -> Result<(bool, bool, bool, bool)> {
        let mut nvenc = false;
        let mut amf = false;
        let mut quicksync = false;

        unsafe {
            let _ = CoInitializeEx(Some(ptr::null_mut()), COINIT_APARTMENTTHREADED);
            let _ = MFStartup(MF_VERSION, 0);
        }

        let activates: *mut *mut Option<windows::Win32::Media::MediaFoundation::IMFActivate> =
            ptr::null_mut();
        let mut count: u32 = 0;

        let hr = unsafe {
            MFTEnumEx(
                MFT_CATEGORY_VIDEO_ENCODER,
                MFT_ENUM_FLAG(0),
                None,
                None,
                activates,
                &mut count,
            )
        };

        if hr.is_ok() && !activates.is_null() {
            for i in 0..count {
                let slot: &mut Option<windows::Win32::Media::MediaFoundation::IMFActivate> =
                    unsafe { &mut (**activates.add(i as usize)) };
                if let Some(activate) = slot.as_mut() {
                    if let Ok(name) = Self::get_activate_name(activate) {
                        let lower = name.to_lowercase();
                        if lower.contains("h264") || lower.contains("hevc") {
                            if lower.contains("nvidia") || lower.contains("nvenc") {
                                nvenc = true;
                            } else if lower.contains("amd") || lower.contains("amf") {
                                amf = true;
                            } else if lower.contains("intel")
                                || lower.contains("quick")
                                || lower.contains("qsv")
                            {
                                quicksync = true;
                            }
                        }
                    }
                }
            }
            for i in 0..count {
                unsafe { ptr::drop_in_place(activates.add(i as usize)); }
            }
            unsafe { CoTaskMemFree(Some(activates as *const _)); }
        }

        unsafe { CoUninitialize(); }
        unsafe { MFShutdown() }?;

        if let Ok(ffmpeg_encoders) = Self::detect_ffmpeg_encoders() {
            let lower = ffmpeg_encoders.to_lowercase();
            if lower.contains("nvenc") {
                nvenc = true;
            }
            if lower.contains("amf") {
                amf = true;
            }
            if lower.contains("qsv") || lower.contains("quick") {
                quicksync = true;
            }
        }

        let any_hw = nvenc || amf || quicksync;
        Ok((any_hw, nvenc, amf, quicksync))
    }

    fn get_activate_name(
        activate: &windows::Win32::Media::MediaFoundation::IMFActivate,
    ) -> Result<String> {
        let mut name: *mut u16 = ptr::null_mut();
        let mut len: u32 = 0;
        let hr = unsafe {
            activate.GetAllocatedString(
                &MFT_FRIENDLY_NAME_Attribute,
                &mut name as *mut *mut u16 as *mut windows::core::PWSTR,
                &mut len,
            )
        };

        if hr.is_ok() && !name.is_null() {
            let slice = unsafe { std::slice::from_raw_parts(name, len as usize) };
            let s = String::from_utf16_lossy(slice);
            unsafe {
                let _ = CoTaskMemFree(Some(name as *const _));
            }
            Ok(s)
        } else {
            anyhow::bail!("GetAllocatedString failed")
        }
    }

    fn detect_ffmpeg_encoders() -> Result<String> {
        let output = std::process::Command::new("ffmpeg")
            .args(["-hide_banner", "-encoders"])
            .output()
            .context("ffmpeg not found or failed to run")?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
