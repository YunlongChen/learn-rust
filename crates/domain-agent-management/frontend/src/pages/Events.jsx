// crates/agent-management/frontend/src/pages/Events.jsx
import { Header } from '../components/Header'

export function Events() {
  const events = [
    { time: '2026-06-07 14:32', type: 'AgentRegistered', agent: 'agent-worker-01', desc: 'Registered successfully', color: '#67c23a' },
    { time: '2026-06-07 14:30', type: 'AgentConnected', agent: 'agent-controller-01', desc: 'Connected', color: '#409eff' },
    { time: '2026-06-07 14:25', type: 'AgentApproved', agent: 'agent-worker-02', desc: 'Approved by admin', color: '#409eff' }
  ]

  return (
    <>
      <Header title="Events" />
      <div className="flex-1 p-6">
        <div className="bg-white rounded-lg border border-gray-200">
          <div className="px-5 py-4 border-b border-gray-200">
            <span className="font-semibold text-gray-800">Lifecycle Events</span>
          </div>
          <div className="p-5">
            {events.map((event, i) => (
              <div key={i} className="flex gap-4 py-3 border-b border-gray-100">
                <span className="text-xs text-gray-400 w-36">{event.time}</span>
                <span className="text-xs font-semibold w-36" style={{ color: event.color }}>{event.type}</span>
                <span className="text-xs text-gray-500 w-36">{event.agent}</span>
                <span className="text-sm text-gray-600">{event.desc}</span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </>
  )
}
