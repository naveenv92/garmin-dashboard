import { BrowserRouter, Routes, Route } from 'react-router-dom'
import { Nav } from './components/Nav'
import { Dashboard } from './pages/Dashboard'
import { Activities } from './pages/Activities'
import { ActivityDetail } from './pages/ActivityDetail'
import { Wellness } from './pages/Wellness'

function App() {
  return (
    <BrowserRouter>
      <div className="flex min-h-screen bg-slate-900">
        <Nav />
        <main className="flex-1 overflow-auto">
          <Routes>
            <Route path="/" element={<Dashboard />} />
            <Route path="/activities" element={<Activities />} />
            <Route path="/activities/:id" element={<ActivityDetail />} />
            <Route path="/wellness" element={<Wellness />} />
          </Routes>
        </main>
      </div>
    </BrowserRouter>
  )
}

export default App
