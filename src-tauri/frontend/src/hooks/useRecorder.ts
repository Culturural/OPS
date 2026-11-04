import { invoke } from '@tauri-apps/api/core'
import type {
  HardwareInfoDto,
  WindowDto,
  RecordingStatusDto,
  RecordingConfig,
  StartRecordingResult,
} from '@/types'

export const getHardwareInfo = async (): Promise<HardwareInfoDto> => {
  return invoke<HardwareInfoDto>('get_hardware_info')
}

export const enumerateWindows = async (): Promise<WindowDto[]> => {
  return invoke<WindowDto[]>('enumerate_windows_cmd')
}

export const getRecordingStatus = async (): Promise<RecordingStatusDto> => {
  return invoke<RecordingStatusDto>('get_recording_status')
}

export const startRecording = async (
  config: RecordingConfig,
): Promise<StartRecordingResult> => {
  return invoke<StartRecordingResult>('start_recording', { config })
}

export const stopRecording = async (): Promise<StartRecordingResult> => {
  return invoke<StartRecordingResult>('stop_recording')
}

export const getAvailableEncoders = async (): Promise<string[]> => {
  return invoke<string[]>('get_available_encoders')
}

export const getAvailableProfiles = async (): Promise<string[]> => {
  return invoke<string[]>('get_available_profiles')
}

export const getAvailablePresets = async (): Promise<string[]> => {
  return invoke<string[]>('get_available_presets')
}
