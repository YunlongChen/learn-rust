// crates/agent-management/frontend/src/pages/Health.jsx
import { Link } from 'react-router-dom'
import { Header } from '../components/Header'

export function Health({ agents }) {
  const avgScore = Math.round(agents.reduce((sum, a) => sum + (a.healthScore || 0), 0) / agents.filter((a) => a.healthScore).length) || 0
  const excellent = agents.filter((a) => a.healthScore >= 90).length
  const good = agents.filter((a) => a.healthScore >= 70 && a.healthScore < 90).length
  const fair = agents.filter((a) => a.healthScore >= 50 && a.healthScore < 70).length

  return (
    <>
      <Header title="Health" />
      <div className="flex-1 p-6">
        <div className="grid grid-cols-4 gap-5 mb-6">
          <div className="bg-white p-5 rounded-lg border border-gray-200">
            <div className="text-sm text-gray-500 mb-2">Avg Health</div>
            <div className="text-3xl font-bold text-gray-800">{avgScore}</div>
          </div>
          <div className="bg-white p-5 rounded-lg border border-gray-200">
            <div className="text-sm text-gray-500 mb-2">Excellent</div>
            <div className="text-3xl font-bold text-success">{excellent}</div>
          </div>
          <div className="bg-white p-5 rounded-lg border border-gray-200">
            <div className="text-sm text-gray-500 mb-2">Good</div>
            <div className="text-3xl font-bold text-primary">{good}</div>
          </div>
          <div className="bg-white p-5 rounded-lg border border-gray-200">
            <div className="text-sm text-gray-500 mb-2">Fair</div>
            <div className="text-3xl font-bold text-warning">{fair}</div>
          </div>
        </div>
        <div className="bg-white rounded-lg border border-gray-200">
          <div className="px-5 py-4 border-b border-gray-200">
            <span className="font-semibold text-gray-800">Agent Health</span>
          </div>
          <table className="w-full">
            <thead>
              <tr className="text-left text-xs text-gray-500 uppercase bg-gray-50">
                <th className="px-5 py-3">Agent</th>
                <th className="px-5 py-3">Score</th>
                <th className="px-5 py-3">Latency</th>
                <th className="px-5 py-3">Jitter</th>
              </tr>
            </thead>
            <tbody>
              {agents.filter((a) => a.healthScore).map((agent) => (
                <tr key={agent.id} className="border-t border-gray-100 hover:bg-gray-50">
                  <td className="px-5 py-4"><Link to={`/agents/${agent.id}`} className="text-primary hover:underline">{agent.name}</Link></td>
                  <td className="px-5 py-4 font-semibold text-success">{agent.healthScore}</td>
                  <td className="px-5 py-4">-</td>
                  <td className="px-5 py-4">-</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </>
  )
}
