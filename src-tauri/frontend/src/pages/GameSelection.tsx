import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { Search, Monitor, RefreshCw } from 'lucide-react'
import { enumerateWindows } from '@/hooks/useRecorder'

export const GameSelection = () => {
  const [windows, setWindows] = useState<any[]>([])
  const [loading, setLoading] = useState(true)
  const [search, setSearch] = useState('')

  useEffect(() => {
    const load = async () => {
      try {
        const ws = await enumerateWindows()
        setWindows(ws)
      } catch (e) {
        console.error('Failed to enumerate windows:', e)
      }
      setLoading(false)
    }
    load()
  }, [])

  const filtered = windows.filter((w) =>
    w.title.toLowerCase().includes(search.toLowerCase())
  )

  const handleSelect = (title: string) => {
    // Store selection and navigate back
    localStorage.setItem('ops_selected_window', title)
    window.location.href = '/'
  }

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="flex flex-col items-center gap-4">
          <div className="w-8 h-8 border-4 border-ops-primary border-t-transparent rounded-full animate-spin"></div>
          <p className="text-slate-400">Scanning windows...</p>
        </div>
      </div>
    )
  }

  return (
    <div className="h-full overflow-y-auto p-8">
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-white mb-2">Game Selection</h1>
        <p className="text-slate-400">Select a window to capture.</p>
      </div>

      <div className="flex gap-4 mb-8">
        <input
          type="text"
          placeholder="Search windows..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="input flex-1"
        />
        <button
          onClick={() => {
            setLoading(true)
            enumerateWindows().then((ws) => setWindows(ws)).finally(() => setLoading(false))
          }}
          className="px-4 py-2 bg-slate-800 text-slate-300 rounded-lg hover:bg-slate-700 flex items-center gap-2"
        >
          <RefreshCw size={16} />
          Refresh
        </button>
      </div>

      <div className="card max-w-2xl">
        <div className="overflow-y-auto max-h-[600px]">
          {filtered.length === 0 ? (
            <div className="p-8 text-center text-slate-500">
              No windows found matching "{search}"
            </div>
          ) : (
            <table className="w-full text-left">
              <thead>
                <tr className="border-b border-slate-800">
                  <th className="py-3 text-slate-400 text-sm font-medium">Window Title</th>
                  <th className="py-3 text-slate-400 text-sm font-medium">PID</th>
                  <th className="py-3 text-slate-400 text-sm font-medium">Action</th>
                </tr>
              </thead>
              <tbody>
                {filtered.map((w) => (
                  <tr key={w.hwnd} className="border-b border-slate-800 hover:bg-slate-800/50 transition-colors">
                    <td className="py-3">
                      <div className="flex items-center gap-3">
                        <div className="w-4 h-4 rounded bg-slate-700 flex items-center justify-center">
                          <span className="text-xs">W</span>
                        </div>
                        <span className="font-medium">{w.title}</span>
                      </div>
                    </td>
                    <td className="py-3 text-slate-400 text-mono">{w.process_id}</td>
                    <td className="py-3">
                      <button
                        onClick={() => handleSelect(w.title)}
                        className="px-3 py-1 bg-ops-primary/10 text-ops-primary rounded text-sm hover:bg-ops-primary/20"
                      >
                        Select
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      </div>

      <div className="mt-8 p-6 bg-slate-900 rounded-xl border border-slate-800">
        <h3 className="text-sm font-semibold text-slate-400 mb-3 uppercase tracking-wider">How it works</h3>
        <ul className="space-y-2 text-sm text-slate-400">
          <li>• Select a window to capture only that application</li>
          <li>• WGC (Windows Graphics Capture) is used for window capture</li>
          <li>• No injection or hooking - anti-cheat safe</li>
          <li>• Minimized windows may capture at reduced frame rate</li>
        </ul>
      </div>
    </div>
  )
}
