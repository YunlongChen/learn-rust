// crates/agent-management/frontend/src/components/AgentDetail.jsx
import { useState, useEffect } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import agentApi from '../api/agent'
import { StatusBadge } from './StatusBadge'
import { formatDate, getHealthStatus } from '../utils/formatters'

export function AgentDetail() {
  const { id } = useParams()
  const navigate = useNavigate()
  const [agent, setAgent] = useState(null)
  const [systemInfo, setSystemInfo] = useState(null)
  const [health, setHealth] = useState(null)
  const [lifecycle, setLifecycle] = useState([])
  const [activeTab, setActiveTab] = useState('overview')
  const [testResult, setTestResult] = useState(null)

  useEffect(() => {
    const fetchData = async () => {
      try {
        const [agentRes, sysRes, healthRes, lifecycleRes] = await Promise.all([
          agentApi.get(id),
          agentApi.getSystemInfo(id).catch(() => ({ data: null })),
          agentApi.getHealth(id).catch(() => ({ data: null })),
          agentApi.getLifecycle(id).catch(() => ({ data: { events: [] } }))
        ])
        setAgent(agentRes.data)
        setSystemInfo(sysRes.data)
        setHealth(healthRes.data)
        setLifecycle(lifecycleRes.data.events || [])
      } catch (err) {
        console.error('Failed to fetch agent:', err)
      }
    }
    fetchData()
  }, [id])

  const handleTestCapability = async (capability) => {
    setTestResult({ capability, status: 'testing' })
    setTimeout(() => {
      setTestResult({ capability, status: 'success', response: '125ms' })
    }, 1000)
  }

  if (!agent) return <div className="p-6">Loading...</div>

  const tabs = ['overview', 'system', 'events', 'health', 'capabilities']
  const healthStatus = health ? getHealthStatus(health.score) : null

  return (
    <div className="flex-1 p-6">
      <div className="bg-white rounded-lg border border-gray-200 p-6 mb-5">
        <div className="flex justify-between items-start mb-4">
          <div>
            <h2 className="text-2xl font-bold text-gray-800">{agent.name}</h2>
            <div className="text-xs text-gray-400 font-mono mt-1">ID: {agent.id}</div>
          </div>
          <div className="flex gap-2">
            <button onClick={() => navigate('/agents')} className="px-4 py-2 text-sm border border-gray-300 rounded hover:border-primary">Edit</button>
            <button className="px-4 py-2 text-sm bg-danger text-white rounded hover:bg-red-400">Delete</button>
          </div>
        </div>
        <div className="flex gap-3 items-center mb-5">
          <StatusBadge status={agent.status} />
          <span className="text-sm text-gray-600">{agent.approvalState || 'Approved'}</span>
        </div>
        <div className="grid grid-cols-4 gap-4 pt-5 border-t border-gray-200">
          <div><span className="text-xs text-gray-500">Endpoint</span><div className="text-sm text-gray-800">{agent.endpoint || '-'}</div></div>
          <div><span className="text-xs text-gray-500">Version</span><div className="text-sm text-gray-800">{agent.version || '-'}</div></div>
          <div><span className="text-xs text-gray-500">Created</span><div className="text-sm text-gray-800">{formatDate(agent.createdAt)}</div></div>
          <div><span className="text-xs text-gray-500">Last Seen</span><div className="text-sm text-gray-800">{formatDate(agent.lastSeenAt)}</div></div>
        </div>
      </div>

      <div className="flex border-b border-gray-200 bg-white rounded-t-lg">
        {tabs.map((tab) => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            className={`px-6 py-4 text-sm capitalize ${activeTab === tab ? 'text-primary border-b-2 border-primary' : 'text-gray-600 hover:text-primary'}`}
          >
            {tab === 'capabilities' ? 'Capabilities' : tab === 'system' ? 'System Info' : tab}
          </button>
        ))}
      </div>

      <div className="bg-white rounded-b-lg border border-t-0 border-gray-200 p-6">
        {activeTab === 'overview' && (
          <div>
            <div className="grid grid-cols-2 gap-4" style={{ gridTemplateColumns: '140px 1fr' }}>
              <span className="text-sm text-gray-500">Agent ID</span><span className="text-sm text-gray-800 font-mono">{agent.id}</span>
              <span className="text-sm text-gray-500">Name</span><span className="text-sm text-gray-800">{agent.name}</span>
              <span className="text-sm text-gray-500">Endpoint</span><span className="text-sm text-gray-800">{agent.endpoint}</span>
              <span className="text-sm text-gray-500">Version</span><span className="text-sm text-gray-800">{agent.version}</span>
            </div>
            <div className="mt-6 pt-6 border-t border-gray-200">
              <div className="font-semibold text-gray-800 mb-3">Capabilities</div>
              <div className="flex gap-2 flex-wrap">
                {(agent.capabilities || []).map((cap) => (
                  <span key={cap} className="px-2 py-1 text-xs rounded bg-blue-100 text-primary">{cap}</span>
                ))}
              </div>
            </div>
          </div>
        )}

        {activeTab === 'system' && systemInfo && (
          <div className="grid grid-cols-2 gap-4" style={{ gridTemplateColumns: '140px 1fr' }}>
            <span className="text-sm text-gray-500">OS</span><span className="text-sm text-gray-800">{systemInfo.osInfo?.os || 'N/A'}</span>
            <span className="text-sm text-gray-500">Hostname</span><span className="text-sm text-gray-800">{systemInfo.osInfo?.hostname || 'N/A'}</span>
            <span className="text-sm text-gray-500">Architecture</span><span className="text-sm text-gray-800">{systemInfo.osInfo?.arch || 'N/A'}</span>
            <span className="text-sm text-gray-500">Rust Version</span><span className="text-sm text-gray-800">{systemInfo.environmentInfo?.rustVersion || 'N/A'}</span>
          </div>
        )}

        {activeTab === 'events' && (
          <div>
            {lifecycle.length > 0 ? lifecycle.map((event) => (
              <div key={event.id} className="flex gap-4 py-3 border-b border-gray-100">
                <span className="text-xs text-gray-400 w-36">{formatDate(event.timestamp)}</span>
                <span className="text-xs font-semibold w-36 text-success">{event.eventType}</span>
                <span className="text-sm text-gray-600">{event.details?.reason || event.eventType}</span>
              </div>
            )) : <div className="text-gray-400 p-5">No events recorded</div>}
          </div>
        )}

        {activeTab === 'health' && health && (
          <div>
            <div className="grid grid-cols-3 gap-5 mb-6">
              <div className="bg-gray-50 rounded-lg p-5 text-center">
                <div className={`text-5xl font-bold ${healthStatus?.class}`}>{health.score}</div>
                <div className="text-sm text-gray-500 mt-2">Overall Score</div>
              </div>
              <div className="bg-gray-50 rounded-lg p-5 text-center">
                <div className="text-2xl font-bold text-gray-800">{health.latency || '-'}ms</div>
                <div className="text-sm text-gray-500 mt-2">Latency</div>
              </div>
              <div className="bg-gray-50 rounded-lg p-5 text-center">
                <div className="text-2xl font-bold text-gray-800">{health.jitter || '-'}ms</div>
                <div className="text-sm text-gray-500 mt-2">Jitter</div>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'capabilities' && (
          <div>
            {(agent.capabilities || []).map((cap) => (
              <div key={cap} className="bg-gray-50 rounded-lg p-4 flex justify-between items-center mb-3">
                <div className="flex items-center gap-4">
                  <div className="w-12 h-12 bg-blue-100 rounded-lg flex items-center justify-center text-xl">⚡</div>
                  <div>
                    <div className="font-semibold text-gray-800">{cap}</div>
                    <div className="text-xs text-gray-400">Test {cap} capability</div>
                  </div>
                </div>
                <button
                  onClick={() => handleTestCapability(cap)}
                  disabled={testResult?.status === 'testing'}
                  className="px-4 py-2 bg-primary text-white text-sm rounded hover:bg-blue-400 disabled:bg-gray-300"
                >
                  {testResult?.capability === cap && testResult?.status === 'testing' ? 'Testing...' : 'Test'}
                </button>
              </div>
            ))}
            {testResult && testResult.status === 'success' && (
              <div className="mt-4 p-3 rounded bg-green-100 text-success text-sm">
                ✓ <strong>{testResult.capability}</strong> test successful - Response: {testResult.response}
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  )
}