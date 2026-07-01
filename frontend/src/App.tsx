import { BrowserRouter, Routes, Route } from 'react-router-dom'
import { Nav } from './components/Nav'
import { UnitsToggle } from './components/UnitsToggle'
import { UnitsProvider } from './context/UnitsContext'
import { Dashboard } from './pages/Dashboard'
import { Activities } from './pages/Activities'
import { ActivityDetail } from './pages/ActivityDetail'
import { Wellness } from './pages/Wellness'

function App() {
  return (
    <UnitsProvider>
      <BrowserRouter>
        <div className="flex min-h-screen bg-slate-900">
          <Nav />
          <main className="flex-1 overflow-auto">
            <div className="flex justify-end px-6 pt-4">
              <UnitsToggle />
            </div>
            <Routes>
              <Route path="/" element={<Dashboard />} />
              <Route path="/activities" element={<Activities />} />
              <Route path="/activities/:id" element={<ActivityDetail />} />
              <Route path="/wellness" element={<Wellness />} />
            </Routes>
          </main>
        </div>
      </BrowserRouter>
    </UnitsProvider>
  )
}

export default App
