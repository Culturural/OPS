import { Link, useLocation } from 'react-router-dom'
import { useEffect, useState } from 'react'
import { getRecordingStatus } from '@/hooks/useRecorder'
import { Monitor, Folder, Settings, Gamepad2, LayoutDashboard } from 'lucide-react'

const Icon = ({ component: Icon, className }: { component: React.ComponentType<any>; className?: string }) => (
  <Icon className={className} />
)

export const Navbar = () => {
  const [recording, setRecording] = useState(false)

  const paths = [
    { path: '/', label: 'Dashboard', icon: Monitor, exact: true },
    { path: '/games', label: 'Game Selection', icon: Gamepad2 },
    { path: '/settings/recording', label: 'Recording Settings', icon: LayoutDashboard },
    { path: '/library', label: 'Recording Library', icon: Folder },
    { path: '/settings/general', label: 'General Settings', icon: Settings },
  ]

  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const status = await getRecordingStatus()
        setRecording(status.is_recording)
      } catch {
        // ignore
      }
    }, 2000)
    return () => clearInterval(interval)
  }, [])

  return (
    <nav className="bg-slate-900 border-b border-slate-800 h-14 flex items-center px-6">
      <div className="flex items-center gap-3">
        <div className="w-8 h-8 bg-ops-primary rounded-lg flex items-center justify-center">
          <span className="text-white font-bold text-lg">O</span>
        </div>
        <span className="text-xl font-bold tracking-tight text-white">OPS Recorder</span>
      </div>

      <div className="ml-auto flex items-center gap-2">
        {paths.map((p) => (
          <Link
            key={p.path}
            to={p.path}
            className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
              location.pathname === p.path ||
              (p.exact && location.pathname === p.path)
                ? 'bg-ops-primary text-white'
                : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800'
            }`}
          >
            <Icon component={p.icon as any} className="w-4 h-4 mr-2" />
            {p.label}
          </Link>
        ))}
      </div>

      <div className="ml-4 flex items-center gap-2 px-4 py-2">
        <span className="text-xs font-mono text-slate-400">
          {recording ? '● REC' : '○ IDLE'}
        </span>
        <div
          className={`w-2 h-2 rounded-full transition-colors ${
            recording ? 'bg-red-500 animate-pulse' : 'bg-slate-500'
          }`}
        />
      </div>
    </nav>
  )
}
