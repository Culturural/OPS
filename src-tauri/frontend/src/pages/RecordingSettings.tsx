import { useEffect, useState } from 'react'
import { getAvailableEncoders, getAvailableProfiles, getAvailablePresets } from '@/hooks/useRecorder'

export const RecordingSettings = () => {
  const [encoders, setEncoders] = useState<string[]>([])
  const [profiles, setProfiles] = useState<string[]>([])
  const [presets, setPresets] = useState<string[]>([])
  const [loading, setLoading] = useState(true)

  const [config, setConfig] = useState({
    fps: 30,
    width: 0,
    height: 0,
    encoder: 'auto',
    crf: 21,
    quality: 21,
    preset: 'veryfast',
    profile: 'balanced',
  })

  useEffect(() => {
    const load = async () => {
      const [e, p, pr] = await Promise.all([
        getAvailableEncoders(),
        getAvailableProfiles(),
        getAvailablePresets(),
      ])
      setEncoders(e)
      setProfiles(p)
      setPresets(pr)
      setLoading(false)
    }
    load()
  }, [])

  return (
    <div className="h-full overflow-y-auto p-8">
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-white mb-2">Recording Settings</h1>
        <p className="text-slate-400">Configure encoding parameters for your recordings.</p>
      </div>

      {loading ? (
        <div className="flex h-full items-center justify-center">
          <div className="flex flex-col items-center gap-4">
            <div className="w-8 h-8 border-4 border-ops-primary border-t-transparent rounded-full animate-spin"></div>
            <p className="text-slate-400">Loading settings...</p>
          </div>
        </div>
      ) : (
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
          {/* Encoder */}
          <div className="card lg:col-span-1">
            <h3 className="text-sm font-semibold text-slate-400 mb-4 uppercase tracking-wider">Encoder</h3>
            <div className="space-y-2">
              {encoders.map((e) => (
                <label key={e} className="cursor-pointer">
                  <input
                    type="radio"
                    name="encoder"
                    checked={config.encoder === e}
                    onChange={(ev) =>
                      setConfig({ ...config, encoder: (ev.target as HTMLInputElement).value })
                    }
                    className="mr-2 rounded text-ops-primary"
                  />
                  {e === 'auto' ? 'Auto (Recommended)' : e.toUpperCase()}
                </label>
              ))}
            </div>
          </div>

          {/* Profile */}
          <div className="card lg:col-span-1">
            <h3 className="text-sm font-semibold text-slate-400 mb-4 uppercase tracking-wider">Quality Profile</h3>
            <div className="space-y-2">
              {profiles.map((p) => {
                const desc =
                  p === 'quality' ? 'Best quality, larger files' :
                  p === 'balanced' ? 'Good balance of quality and size' :
                  'Smallest files, acceptable quality'
                return (
                  <label key={p} className="cursor-pointer block">
                    <input
                      type="radio"
                      name="profile"
                      checked={config.profile === p}
                      onChange={(ev) =>
                        setConfig({ ...config, profile: (ev.target as HTMLInputElement).value })
                      }
                      className="mr-2 rounded text-ops-primary"
                    />
                    <div className="flex justify-between">
                      <span>{p.charAt(0).toUpperCase() + p.slice(1)}</span>
                      <span className="text-xs text-slate-500">{desc}</span>
                    </div>
                  </label>
                )
              })}
            </div>
          </div>

          {/* Preset */}
          <div className="card lg:col-span-1">
            <h3 className="text-sm font-semibold text-slate-400 mb-4 uppercase tracking-wider">Preset</h3>
            <div className="space-y-2">
              {presets.map((p) => (
                <label key={p} className="cursor-pointer block">
                  <input
                    type="radio"
                    name="preset"
                    checked={config.preset === p}
                    onChange={(ev) =>
                      setConfig({ ...config, preset: (ev.target as HTMLInputElement).value })
                    }
                    className="mr-2 rounded text-ops-primary"
                  />
                  {p}
                </label>
              ))}
            </div>
          </div>

          {/* Advanced */}
          <div className="card lg:col-span-2">
            <h3 className="text-sm font-semibold text-slate-400 mb-4 uppercase tracking-wider">Advanced Settings</h3>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <label className="block text-xs text-slate-400 mb-1">FPS</label>
                <input
                  type="number"
                  value={config.fps}
                  onChange={(e) =>
                    setConfig({ ...config, fps: parseInt(e.target.value) || 30 })
                  }
                  className="input w-full"
                  min="1" max="240"
                />
              </div>
              <div>
                <label className="block text-xs text-slate-400 mb-1">CRF (libx264 quality, 0-51)</label>
                <input
                  type="number"
                  value={config.crf}
                  onChange={(e) =>
                    setConfig({ ...config, crf: parseInt(e.target.value) || 21 })
                  }
                  className="input w-full"
                  min="0" max="51"
                />
              </div>
              <div>
                <label className="block text-xs text-slate-400 mb-1">Quality (Hardware encoder)</label>
                <input
                  type="number"
                  value={config.quality}
                  onChange={(e) =>
                    setConfig({ ...config, quality: parseInt(e.target.value) || 21 })
                  }
                  className="input w-full"
                  min="0" max="51"
                />
              </div>
              <div>
                <label className="block text-xs text-slate-400 mb-1">Width (0 = auto)</label>
                <input
                  type="number"
                  value={config.width}
                  onChange={(e) =>
                    setConfig({ ...config, width: parseInt(e.target.value) || 0 })
                  }
                  className="input w-full"
                  min="0"
                />
              </div>
              <div>
                <label className="block text-xs text-slate-400 mb-1">Height (0 = auto)</label>
                <input
                  type="number"
                  value={config.height}
                  onChange={(e) =>
                    setConfig({ ...config, height: parseInt(e.target.value) || 0 })
                  }
                  className="input w-full"
                  min="0"
                />
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
