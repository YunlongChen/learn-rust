// crates/agent-management/frontend/src/components/DenyModal.jsx
import { useState } from 'react'

export function DenyModal({ isOpen, agent, onClose, onDeny }) {
  const [reason, setReason] = useState('')

  if (!isOpen || !agent) return null

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg w-full max-w-md p-6">
        <div className="w-16 h-16 rounded-full bg-red-100 mx-auto flex items-center justify-center text-3xl text-danger mb-4">✕</div>
        <h3 className="text-lg font-semibold text-gray-800 text-center mb-2">Deny Agent</h3>
        <p className="text-gray-600 text-center mb-4">Deny "{agent.name}"? Please provide a reason.</p>
        <textarea
          className="w-full px-3 py-2 border border-gray-300 rounded text-sm mb-4 focus:outline-none focus:border-primary"
          rows="3"
          placeholder="Denial reason..."
          value={reason}
          onChange={(e) => setReason(e.target.value)}
        />
        <div className="flex justify-center gap-3">
          <button onClick={onClose} className="px-4 py-2 text-sm border border-gray-300 rounded">Cancel</button>
          <button onClick={() => { onDeny(agent.id, reason); setReason(''); onClose() }} className="px-4 py-2 text-sm bg-danger text-white rounded hover:bg-red-400">Deny</button>
        </div>
      </div>
    </div>
  )
}
