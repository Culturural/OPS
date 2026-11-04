export interface HardwareInfoDto {
  cpu_manufacturer: string
  cpu_model: string
  cpu_cores: number
  cpu_threads: number
  total_ram_mb: number
  gpu_manufacturer: string | null
  gpu_model: string | null
  gpu_adapter_count: number
  windows_version: string
  windows_build: number
  display_width: number
  display_height: number
  hardware_encoder_available: boolean
  nvenc_available: boolean
  amf_available: boolean
  quicksync_available: boolean
}

export interface WindowDto {
  hwnd: number
  title: string
  process_id: number
}

export interface RecordingStatusDto {
  is_recording: boolean
  frames_captured: number
  frames_encoded: number
  frames_dropped: number
  capture_fps: number
  encode_fps: number
  avg_encode_ms: number
  cpu_utilization: number
  elapsed_seconds: number
  output_path: string | null
  encoder: string
}

export interface RecordingConfig {
  output_path: string
  fps: number
  width: number
  height: number
  encoder: string
  crf?: number
  quality?: number
  preset?: string
  profile: string
  window_title?: string
}

export interface StartRecordingResult {
  success: boolean
  message: string
}

export interface RecordedFile {
  name: string
  path: string
  size: number
  duration?: number
}
