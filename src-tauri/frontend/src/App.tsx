import { BrowserRouter, Routes, Route } from 'react-router-dom'
import { Navbar } from '@/components/Navbar'
import { Dashboard } from '@/pages/Dashboard'
import { GameSelection } from '@/pages/GameSelection'
import { RecordingSettings } from '@/pages/RecordingSettings'
import { RecordingLibrary } from '@/pages/RecordingLibrary'
import { GeneralSettings } from '@/pages/GeneralSettings'

function App() {
  return (
    <BrowserRouter>
      <div className="min-h-screen bg-slate-950 text-slate-100">
        <Navbar />
        <main className="w-full">
          <Routes>
            <Route path="/" element={<Dashboard />} />
            <Route path="/dashboard" element={<Dashboard />} />
            <Route path="/games" element={<GameSelection />} />
            <Route path="/settings/recording" element={<RecordingSettings />} />
            <Route path="/library" element={<RecordingLibrary />} />
            <Route path="/settings/general" element={<GeneralSettings />} />
            <Route path="*" element={<Dashboard />} />
          </Routes>
        </main>
      </div>
    </BrowserRouter>
  )
}

export default App
