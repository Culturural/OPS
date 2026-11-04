import { useEffect, useState } from 'react'
import { FileText, Folder, Play, ExternalLink } from 'lucide-react'

export const RecordingLibrary = () => {
  const [recordings, setRecordings] = useState<any[]>([])

  useEffect(() => {
    // Scan recordings directory
    const dir = 'recordings'
    // For now, show empty state - in production this would use Tauri fs API
    setRecordings([])
  }, [])

  const openFile = (path: string) => {
    // Would use Tauri shell.open
    console.log('Open:', path)
  }

  const openFolder = () => {
    // Would use Tauri shell.open
    console.log('Open folder')
  }

  return (
    <div className="h-full overflow-y-auto p-8">
      <div className="flex justify-between items-center mb-8">
        <div>
          <h1 className="text-3xl font-bold text-white mb-2">Recording Library</h1>
          <p className="text-slate-400">Your recorded gameplay videos.</p>
        </div>
        <button
          onClick={openFolder}
          className="px-4 py-2 bg-slate-800 text-slate-300 rounded-lg hover:bg-slate-700 flex items-center gap-2"
        >
          <Folder size={18} />
          Open Folder
        </button>
      </div>

      <div className="card">
        {recordings.length === 0 ? (
          <div className="p-12 text-center">
            <FileText className="w-16 h-16 text-slate-600 mx-auto mb-4" />
            <h3 className="text-lg font-medium text-white mb-2">No recordings yet</h3>
            <p className="text-slate-400">Start recording to see your files here.</p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b border-slate-800">
                  <th className="py-3 text-slate-400 text-sm font-medium">Name</th>
                  <th className="py-3 text-slate-400 text-sm font-medium">Size</th>
                  <th className="py-3 text-slate-400 text-sm font-medium">Duration</th>
                  <th className="py-3 text-slate-400 text-sm font-medium">Actions</th>
                </tr>
              </thead>
              <tbody>
                {recordings.map((r) => (
                  <tr key={r.name} className="border-b border-slate-800 hover:bg-slate-800/50 transition-colors">
                    <td className="py-3">
                      <div className="flex items-center gap-3">
                        <FileText className="w-4 h-4 text-slate-500" />
                        <span className="font-medium">{r.name}</span>
                      </div>
                    </td>
                    <td className="py-3 text-slate-400 text-mono">
                      {(r.size / 1024 / 1024).toFixed(2)} MB
                    </td>
                    <td className="py-3 text-slate-400">
                      {r.duration ? r.duration.toFixed(1) + 's' : '—'}
                    </td>
                    <td className="py-3">
                      <div className="flex gap-2">
                        <button
                          onClick={() => openFile(r.path)}
                          className="px-3 py-1 bg-slate-800 text-slate-300 rounded text-sm hover:bg-slate-700"
                        >
                          <Play size={14} />
                          Play
                        </button>
                        <button
                          className="px-3 py-1 bg-slate-800 text-slate-300 rounded text-sm hover:bg-slate-700"
                        >
                          <ExternalLink size={14} />
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  )
}
