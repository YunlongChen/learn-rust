// crates/agent-management/frontend/src/components/ApproveModal.jsx
export function ApproveModal({ isOpen, agent, onClose, onApprove }) {
  if (!isOpen || !agent) return null

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg w-full max-w-sm text-center p-6">
        <div className="w-16 h-16 rounded-full bg-green-100 mx-auto flex items-center justify-center text-3xl text-success mb-4">✓</div>
        <h3 className="text-lg font-semibold text-gray-800 mb-2">Approve Agent</h3>
        <p className="text-gray-600 mb-6">Approve "{agent.name}"? It will transition from Pending to Authorized.</p>
        <div className="flex justify-center gap-3">
          <button onClick={onClose} className="px-4 py-2 text-sm border border-gray-300 rounded hover:border-primary">Cancel</button>
          <button onClick={() => { onApprove(agent.id); onClose() }} className="px-4 py-2 text-sm bg-success text-white rounded hover:bg-green-400">Approve</button>
        </div>
      </div>
    </div>
  )
}
