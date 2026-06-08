// crates/agent-management/frontend/src/components/DeleteModal.jsx
export function DeleteModal({ isOpen, agent, onClose, onDelete }) {
  if (!isOpen || !agent) return null

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg w-full max-w-sm text-center p-6">
        <div className="w-16 h-16 rounded-full bg-red-100 mx-auto flex items-center justify-center text-3xl text-danger mb-4">!</div>
        <h3 className="text-lg font-semibold text-gray-800 mb-2">Delete Agent</h3>
        <p className="text-gray-600 mb-6">Are you sure you want to delete "{agent.name}"? This cannot be undone.</p>
        <div className="flex justify-center gap-3">
          <button onClick={onClose} className="px-4 py-2 text-sm border border-gray-300 rounded">Cancel</button>
          <button onClick={() => { onDelete(agent.id); onClose() }} className="px-4 py-2 text-sm bg-danger text-white rounded hover:bg-red-400">Delete</button>
        </div>
      </div>
    </div>
  )
}
