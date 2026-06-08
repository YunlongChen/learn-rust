// crates/agent-management/frontend/src/pages/Dashboard.jsx
import { Link } from 'react-router-dom'
import { StatusBadge } from '../components/StatusBadge'

export function Dashboard({ agents }) {
  const stats = {
    total: agents.length,
    online: agents.filter((a) => ['connected', 'registered'].includes(a.status)).length,
    pending: agents.filter((a) => a.status === 'pending').length,
    avgHealth: Math.round(agents.reduce((sum, a) => sum + (a.healthScore || 0), 0) / agents.filter((a) => a.healthScore).length) || 0
  }

  return (
    <div className="flex-1 p-6">
      <div className="grid grid-cols-4 gap-5 mb-6">
        <div className="bg-white p-5 rounded-lg border border-gray-200">
          <div className="text-sm text-gray-500 mb-2">Total Agents</div>
          <div className="text-3xl font-bold text-gray-800">{stats.total}</div>
        </div>
        <div className="bg-white p-5 rounded-lg border border-gray-200">
          <div className="text-sm text-gray-500 mb-2">Online</div>
          <div className="text-3xl font-bold text-success">{stats.online}</div>
        </div>
        <div className="bg-white p-5 rounded-lg border border-gray-200">
          <div className="text-sm text-gray-500 mb-2">Pending</div>
          <div className="text-3xl font-bold text-warning">{stats.pending}</div>
        </div>
        <div className="bg-white p-5 rounded-lg border border-gray-200">
          <div className="text-sm text-gray-500 mb-2">Avg Health</div>
          <div className="text-3xl font-bold text-gray-800">{stats.avgHealth}</div>
        </div>
      </div>
      <div className="bg-white rounded-lg border border-gray-200">
        <div className="px-5 py-4 border-b border-gray-200 flex justify-between items-center">
          <span className="font-semibold text-gray-800">Recent Agents</span>
          <Link to="/agents" className="text-sm text-primary hover:underline">View all</Link>
        </div>
        <table className="w-full">
          <thead>
            <tr className="text-left text-xs text-gray-500 uppercase bg-gray-50">
              <th className="px-5 py-3">Name</th>
              <th className="px-5 py-3">Status</th>
              <th className="px-5 py-3">Health</th>
            </tr>
          </thead>
          <tbody>
            {agents.slice(0, 5).map((agent) => (
              <tr key={agent.id} className="border-t border-gray-100 hover:bg-gray-50">
                <td className="px-5 py-4"><Link to={`/agents/${agent.id}`} className="text-primary hover:underline">{agent.name}</Link></td>
                <td className="px-5 py-4"><StatusBadge status={agent.status} /></td>
                <td className="px-5 py-4">{agent.healthScore || '-'}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  )
}
