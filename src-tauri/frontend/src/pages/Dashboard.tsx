import { useEffect, useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import {
  Play,
  StopCircle,
  RefreshCw,
  ChevronRight,
  Video,
  Clock,
  HardDrive,
  MicOff,
} from 'lucide-react'
import { getRecordingStatus, startRecording, stopRecording, getHardwareInfo } from '@/hooks/useRecorder'
import type { RecordingConfig } from '@/types'

export const Dashboard = () => {
  const navigate = useNavigate()
  const [status, setStatus] = useState<any>({
    is_recording: false,
    frames_captured: 0,
    frames_encoded: 0,
    frames_dropped: 0,
    capture_fps: 0,
    encode_fps: 0,
    avg_encode_ms: 0,
    cpu_utilization: 0,
    elapsed_seconds: 0,
    output_path: null,
    encoder: 'idle',
  })
  const [hardware, setHardware] = useState<any>(null)
  const [loading, setLoading] = useState(true)
  const [outputPath, setOutputPath] = useState('recordings/output.mp4')
  const [selectedSource, setSelectedSource] = useState('Desktop (Primary Monitor)')
  const [selectedEncoder, setSelectedEncoder] = useState('auto')
  const [selectedProfile, setSelectedProfile] = useState('balanced')

  useEffect(() => {
    const load = async () => {
      try {
        const [hw] = await Promise.all([getHardwareInfo()])
        setHardware(hw)
      } catch (e) {
        console.error('Failed to load hardware info:', e)
      }
      setLoading(false)
    }
    load()
  }, [])

  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const s = await getRecordingStatus()
        setStatus(s)
      } catch {
        // ignore
      }
    }, 1000)
    return () => clearInterval(interval)
  }, [])

  const handleStart = async () => {
    const config: RecordingConfig = {
      output_path: outputPath,
      fps: 30,
      width: 0,
      height: 0,
      encoder: selectedEncoder,
      profile: selectedProfile,
      window_title: undefined,
    }
    try {
      const result = await startRecording(config)
      if (result.success) {
        navigate('/dashboard')
      } else {
        alert(result.message)
      }
    } catch (e) {
      alert(`Failed to start recording: ${e}`)
    }
  }

  const handleStop = async () => {
    try {
      await stopRecording()
    } catch (e) {
      alert(`Failed to stop recording: ${e}`)
    }
  }

  const formatTime = (seconds: number) => {
    const m = Math.floor(seconds / 60)
    const s = Math.floor(seconds % 60)
    return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`
  }

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="flex flex-col items-center gap-4">
          <div className="w-8 h-8 border-4 border-ops-primary border-t-transparent rounded-full animate-spin"></div>
          <p className="text-slate-400">Initializing...</p>
        </div>
      </div>
    )
  }

  return (
    <div className="h-full overflow-y-auto p-8">
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-white mb-2">Dashboard</h1>
        <p className="text-slate-400">Record your gameplay with minimal performance impact.</p>
      </div>

      {/* Status Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
        <div className="stat-card">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-slate-400 text-sm">Capture FPS</p>
              <p className="text-2xl font-mono font-bold">{status.capture_fps.toFixed(1)}</p>
            </div>
            <Video className="w-6 h-6 text-ops-primary" />
          </div>
        </div>
        <div className="stat-card">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-slate-400 text-sm">Encode FPS</p>
              <p className="text-2xl font-mono font-bold">{status.encode_fps.toFixed(1)}</p>
            </div>
            <RefreshCw className="w-6 h-6 text-ops-secondary" />
          </div>
        </div>
        <div className="stat-card">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-slate-400 text-sm">CPU Usage</p>
              <p className="text-2xl font-mono font-bold">{status.cpu_utilization.toFixed(1)}%</p>
            </div>
            <HardDrive className="w-6 h-6 text-ops-accent" />
          </div>
        </div>
        <div className="stat-card">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-slate-400 text-sm">Encoder</p>
              <p className="text-xl font-mono font-bold text-ops-accent">
                {status.encoder === 'idle' ? 'Idle' : status.encoder}
              </p>
            </div>
            <MicOff className="w-6 h-6 text-slate-500" />
          </div>
        </div>
      </div>

      {/* Recording Controls */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        {/* Left: Controls */}
        <div className="lg:col-span-1">
          <div className="card">
            <h2 className="text-lg font-semibold mb-4">Recording Controls</h2>

            <div className="flex flex-col gap-4">
              {!status.is_recording ? (
                <button
                  onClick={handleStart}
                  disabled={loading}
                  className="btn-primary w-full py-4 flex items-center justify-center gap-2 text-lg"
                >
                  <Play className="w-6 h-6" />
                  Start Recording
                </button>
              ) : (
                <button
                  onClick={handleStop}
                  className="btn-danger w-full py-4 flex items-center justify-center gap-2 text-lg"
                >
                  <StopCircle className="w-6 h-6" />
                  Stop Recording
                </button>
              )}
            </div>

            {status.is_recording && (
              <div className="mt-4 p-4 bg-slate-800 rounded-lg border border-slate-700">
                <div className="flex justify-between mb-2">
                  <span className="text-sm text-slate-400">Status</span>
                  <span className="text-sm font-bold text-green-400">Recording</span>
                </div>
                <div className="flex justify-between mb-2">
                  <span className="text-sm text-slate-400">Elapsed</span>
                  <span className="text-sm font-mono">{formatTime(status.elapsed_seconds)}</span>
                </div>
                <div className="flex justify-between mb-2">
                  <span className="text-sm text-slate-400">Output</span>
                  <span className="text-sm font-mono truncate">{status.output_path}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-sm text-slate-400">Frames</span>
                  <span className="text-sm font-mono">
                    {status.frames_encoded} / {status.frames_captured}
                  </span>
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Center: Preview / Source Selection */}
        <div className="lg:col-span-2">
          <div className="card">
            <div className="flex justify-between items-start mb-4">
              <div>
                <h2 className="text-lg font-semibold">Capture Source</h2>
                <p className="text-slate-400 text-sm">
                  {selectedSource} • {selectedProfile.toUpperCase()} profile
                </p>
              </div>
              <Link to="/games" className="text-sm text-ops-primary hover:text-ops-secondary flex items-center gap-1">
                Change <ChevronRight size={14} />
              </Link>
            </div>

            <div className="bg-slate-950 rounded-xl border border-slate-800 aspect-video relative overflow-hidden">
              <div className="absolute inset-0 flex items-center justify-center">
                <div className="text-center">
                  <div className="w-16 h-16 bg-slate-800 rounded-full flex items-center justify-center mx-auto mb-4">
                    <Play className="w-8 h-8 text-slate-400" />
                  </div>
                  <p className="text-slate-500 text-sm">No active recording</p>
                  <p className="text-xs text-slate-600 mt-1">
                    {status.is_recording ? 'Recording in progress...' : 'Click Start Recording to begin'}
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Quick Settings */}
      <div className="mt-8 grid grid-cols-1 md:grid-cols-3 gap-6">
        <div className="card">
          <h3 className="text-sm font-semibold text-slate-400 mb-3 uppercase tracking-wider">Source</h3>
          <div className="space-y-2">
            <button
              onClick={() => setSelectedSource('Desktop (Primary Monitor)')}
              className={`w-full text-left px-4 py-2 rounded-lg transition-colors ${
                selectedSource === 'Desktop' ? 'bg-ops-primary/20 text-ops-primary border border-ops-primary/50' : 'bg-slate-800 text-slate-300 hover:bg-slate-700'
              }`}
            >
              Desktop (Primary Monitor)
            </button>
            <button
              onClick={() => navigate('/games')}
              className="w-full text-left px-4 py-2 rounded-lg bg-slate-800 text-slate-300 hover:bg-slate-700"
            >
              Select Window / Game
            </button>
          </div>
        </div>
        <div className="card">
          <h3 className="text-sm font-semibold text-slate-400 mb-3 uppercase tracking-wider">Profile</h3>
          <div className="space-y-2">
            {['quality', 'balanced', 'small'].map((p) => (
              <button
                key={p}
                onClick={() => setSelectedProfile(p)}
                className={`w-full text-left px-4 py-2 rounded-lg transition-colors ${
                  selectedProfile === p ? 'bg-ops-primary/20 text-ops-primary border border-ops-primary/50' : 'bg-slate-800 text-slate-300 hover:bg-slate-700'
                }`}
              >
                {p.charAt(0).toUpperCase() + p.slice(1)}
              </button>
            ))}
          </div>
        </div>
        <div className="card">
          <h3 className="text-sm font-semibold text-slate-400 mb-3 uppercase tracking-wider">Output</h3>
          <div className="space-y-2">
            <div className="flex items-center gap-2">
              <input
                type="text"
                value={outputPath}
                onChange={(e) => setOutputPath(e.target.value)}
                className="input flex-1"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
