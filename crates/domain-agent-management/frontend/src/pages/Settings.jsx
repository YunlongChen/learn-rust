// crates/agent-management/frontend/src/pages/Settings.jsx
import { Header } from '../components/Header'

export function Settings() {
  return (
    <>
      <Header title="Settings" />
      <div className="flex-1 p-6">
        <div className="bg-white rounded-lg border border-gray-200 p-5">
          <h3 className="font-semibold text-gray-800 mb-4">General Settings</h3>
          <div className="flex justify-between items-center py-3 border-b border-gray-100">
            <span className="text-sm text-gray-600">Auto-approve agents</span>
            <span className="text-sm text-gray-400">OFF</span>
          </div>
          <div className="flex justify-between items-center py-3 border-b border-gray-100">
            <span className="text-sm text-gray-600">Health monitoring</span>
            <span className="text-sm text-success">ON</span>
          </div>
        </div>
      </div>
    </>
  )
}
