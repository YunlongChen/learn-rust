// crates/agent-management/frontend/src/components/AgentTable.jsx
import { StatusBadge } from './StatusBadge'
import { Link } from 'react-router-dom'

export function AgentTable({ agents, onApprove, onDeny, onDelete }) {
  const getActions = (agent) => {
    switch (agent.status) {
      case 'pending':
        return (
          <>
            <button onClick={() => onApprove(agent)} className="px-3 py-1 text-xs bg-success text-white rounded hover:bg-green-400 mr-2">Approve</button>
            <button onClick={() => onDeny(agent)} className="px-3 py-1 text-xs bg-danger text-white rounded hover:bg-red-400">Deny</button>
          </>
        )
      case 'connected':
      case 'registered':
        return (
          <>
            <Link to={`/agents/${agent.id}`} className="px-3 py-1 text-xs border border-gray-300 rounded hover:border-primary hover:text-primary mr-2">Edit</Link>
            <button onClick={() => onDelete(agent)} className="px-3 py-1 text-xs border border-gray-300 rounded hover:border-danger hover:text-danger">Delete</button>
          </>
        )
      default:
        return <Link to={`/agents/${agent.id}`} className="px-3 py-1 text-xs border border-gray-300 rounded hover:border-primary hover:text-primary">Edit</Link>
    }
  }

  return (
    <table className="w-full">
      <thead>
        <tr className="text-left text-xs text-gray-500 uppercase bg-gray-50">
          <th className="px-5 py-3">Name</th>
          <th className="px-5 py-3">Status</th>
          <th className="px-5 py-3">Health</th>
          <th className="px-5 py-3">Last Seen</th>
          <th className="px-5 py-3">Actions</th>
        </tr>
      </thead>
      <tbody>
        {agents.map((agent) => (
          <tr key={agent.id} className="border-t border-gray-100 hover:bg-gray-50">
            <td className="px-5 py-4">
              <Link to={`/agents/${agent.id}`} className="text-primary hover:underline">{agent.name}</Link>
            </td>
            <td className="px-5 py-4"><StatusBadge status={agent.status} /></td>
            <td className="px-5 py-4">{agent.healthScore || '-'}</td>
            <td className="px-5 py-4 text-gray-500 text-sm">{agent.lastSeenAt || '-'}</td>
            <td className="px-5 py-4">{getActions(agent)}</td>
          </tr>
        ))}
      </tbody>
    </table>
  )
}