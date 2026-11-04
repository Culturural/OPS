import { useEffect, useState } from 'react'
import { getHardwareInfo } from '@/hooks/useRecorder'
import { Cpu, HardDrive, Settings, GitBranch } from 'lucide-react'

export const GeneralSettings = () => {
  const [hardware, setHardware] = useState<any>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    const load = async () => {
      try {
        const hw = await getHardwareInfo()
        setHardware(hw)
      } catch (e) {
        console.error('Failed to load hardware info:', e)
      }
      setLoading(false)
    }
    load()
  }, [])

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="flex flex-col items-center gap-4">
          <div className="w-8 h-8 border-4 border-ops-primary border-t-transparent rounded-full animate-spin"></div>
          <p className="text-slate-400">Loading system info...</p>
        </div>
      </div>
    )
  }

  return (
    <div className="h-full overflow-y-auto p-8">
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-white mb-2">General Settings</h1>
        <p className="text-slate-400">System information and application settings.</p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div className="card">
          <h3 className="text-sm font-semibold text-slate-400 mb-4 uppercase tracking-wider">System</h3>
          <div className="space-y-3">
            <div className="flex justify-between items-center p-3 bg-slate-800 rounded border border-slate-700">
              <span className="text-sm text-slate-400">OS</span>
              <span className="text-sm font-mono">{hardware?.windows_version || '—'} (Build {hardware?.windows_build || '—'})</span>
            </div>
            <div className="flex justify-between items-center p-3 bg-slate-800 rounded border border-slate-700">
              <span className="text-sm text-slate-400">Display</span>
              <span className="text-sm font-mono">{hardware?.display_width || '—'} × {hardware?.display_height || '—'}</span>
            </div>
            <div className="flex justify-between items-center p-3 bg-slate-800 rounded border border-slate-700">
              <span className="text-sm text-slate-400">RAM</span>
              <span className="text-sm font-mono">{hardware?.total_ram_mb || '—'} MB</span>
            </div>
          </div>
        </div>

        <div className="card">
          <h3 className="text-sm font-semibold text-slate-400 mb-4 uppercase tracking-wider">CPU</h3>
          <div className="space-y-3">
            <div className="flex justify-between items-center p-3 bg-slate-800 rounded border border-slate-700">
              <span className="text-sm text-slate-400">Manufacturer</span>
              <span className="text-sm font-mono">{hardware?.cpu_manufacturer || '—'}</span>
            </div>
            <div className="flex justify-between items-center p-3 bg-slate-800 rounded border border-slate-700">
              <span className="text-sm text-slate-400">Model</span>
              <span className="text-sm font-mono truncate">{hardware?.cpu_model || '—'}</span>
            </div>
            <div className="flex justify-between items-center p-3 bg-slate-800 rounded border border-slate-700">
              <span className="text-sm text-slate-400">Cores</span>
              <span className="text-sm font-mono">{hardware?.cpu_cores || '—'}</span>
            </div>
            <div className="flex justify-between items-center p-3 bg-slate-800 rounded border border-slate-700">
              <span className="text-sm text-slate-400">Threads</span>
              <span className="text-sm font-mono">{hardware?.cpu_threads || '—'}</span>
            </div>
          </div>
        </div>

        <div className="card">
          <h3 className="text-sm font-semibold text-slate-400 mb-4 uppercase tracking-wider">GPU</h3>
          <div className="space-y-3">
            <div className="flex justify-between items-center p-3 bg-slate-800 rounded border border-slate-700">
              <span className="text-sm text-slate-400">Manufacturer</span>
              <span className="text-sm font-mono">{hardware?.gpu_manufacturer || '—'}</span>
            </div>
            <div className="flex justify-between items-center p-3 bg-slate-800 rounded border border-slate-700">
              <span className="text-sm text-slate-400">Model</span>
              <span className="text-sm font-mono truncate">{hardware?.gpu_model || '—'}</span>
            </div>
            <div className="flex justify-between items-center p-3 bg-slate-800 rounded border border-slate-700">
              <span className="text-sm text-slate-400">Adapters</span>
              <span className="text-sm font-mono">{hardware?.gpu_adapter_count || '—'}</span>
            </div>
            <div className="flex justify-between items-center p-3 bg-slate-800 rounded border border-slate-700">
              <span className="text-sm text-slate-400">NVENC</span>
              <span className={`text-sm font-mono ${hardware?.nvenc_available ? 'text-green-400' : 'text-red-400'}`}>
                {hardware?.nvenc_available ? 'Available' : 'Not Available'}
              </span>
            </div>
            <div className="flex justify-between items-center p-3 bg-slate-800 rounded border border-slate-700">
              <span className="text-sm text-slate-400">AMF</span>
              <span className={`text-sm font-mono ${hardware?.amf_available ? 'text-green-400' : 'text-red-400'}`}>
                {hardware?.amf_available ? 'Available' : 'Not Available'}
              </span>
            </div>
            <div className="flex justify-between items-center p-3 bg-slate-800 rounded border border-slate-700">
              <span className="text-sm text-slate-400">QuickSync</span>
              <span className={`text-sm font-mono ${hardware?.quicksync_available ? 'text-green-400' : 'text-red-400'}`}>
                {hardware?.quicksync_available ? 'Available' : 'Not Available'}
              </span>
            </div>
          </div>
        </div>
      </div>

      <div className="mt-8 card">
        <h3 className="text-sm font-semibold text-slate-400 mb-4 uppercase tracking-wider">Application</h3>
        <div className="space-y-3">
          <div className="flex justify-between items-center p-3 bg-slate-800 rounded border border-slate-700">
            <span className="text-sm text-slate-400">Version</span>
            <span className="text-sm font-mono">0.1.0</span>
          </div>
          <div className="flex justify-between items-center p-3 bg-slate-800 rounded border border-slate-700">
            <span className="text-sm text-slate-400">Build</span>
            <span className="text-sm font-mono">Windows x64</span>
          </div>
        </div>
      </div>
    </div>
  )
}
