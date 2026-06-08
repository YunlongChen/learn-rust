import { BrowserRouter, Routes, Route } from 'react-router-dom'
import { Layout } from './components/Layout'
import { Dashboard } from './pages/Dashboard'
import { Agents } from './pages/Agents'
import { Events } from './pages/Events'
import { Health } from './pages/Health'
import { Settings } from './pages/Settings'
import { AgentDetail } from './components/AgentDetail'
import { useAgents } from './hooks/useAgents'

function App() {
  const { agents } = useAgents()

  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Layout />}>
          <Route index element={<Dashboard agents={agents} />} />
          <Route path="agents" element={<Agents />} />
          <Route path="agents/:id" element={<AgentDetail />} />
          <Route path="events" element={<Events />} />
          <Route path="health" element={<Health agents={agents} />} />
          <Route path="settings" element={<Settings />} />
        </Route>
      </Routes>
    </BrowserRouter>
  )
}

export default App
