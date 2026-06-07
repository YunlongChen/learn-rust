// crates/agent-management/frontend/src/components/CreateAgentModal.jsx
import { useState } from 'react'

export function CreateAgentModal({ isOpen, onClose, onSubmit }) {
  const [formData, setFormData] = useState({
    name: '',
    endpoint: '',
    version: '',
    description: ''
  })

  const handleSubmit = (e) => {
    e.preventDefault()
    onSubmit(formData)
    setFormData({ name: '', endpoint: '', version: '', description: '' })
    onClose()
  }

  if (!isOpen) return null

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg w-full max-w-md">
        <div className="px-5 py-4 border-b border-gray-200 flex justify-between items-center">
          <span className="font-semibold text-gray-800">Create Agent</span>
          <button onClick={onClose} className="w-8 h-8 rounded hover:bg-gray-100 text-gray-400">×</button>
        </div>
        <form onSubmit={handleSubmit}>
          <div className="p-5 space-y-4">
            <div>
              <label className="block text-sm text-gray-600 mb-1">Name *</label>
              <input
                type="text"
                required
                className="w-full px-3 py-2 border border-gray-300 rounded text-sm focus:outline-none focus:border-primary"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              />
            </div>
            <div>
              <label className="block text-sm text-gray-600 mb-1">Endpoint *</label>
              <input
                type="text"
                required
                className="w-full px-3 py-2 border border-gray-300 rounded text-sm focus:outline-none focus:border-primary"
                value={formData.endpoint}
                onChange={(e) => setFormData({ ...formData, endpoint: e.target.value })}
              />
            </div>
            <div>
              <label className="block text-sm text-gray-600 mb-1">Version</label>
              <input
                type="text"
                className="w-full px-3 py-2 border border-gray-300 rounded text-sm focus:outline-none focus:border-primary"
                value={formData.version}
                onChange={(e) => setFormData({ ...formData, version: e.target.value })}
              />
            </div>
          </div>
          <div className="px-5 py-4 border-t border-gray-200 flex justify-end gap-3">
            <button type="button" onClick={onClose} className="px-4 py-2 text-sm border border-gray-300 rounded hover:border-primary hover:text-primary">Cancel</button>
            <button type="submit" className="px-4 py-2 text-sm bg-primary text-white rounded hover:bg-blue-400">Create</button>
          </div>
        </form>
      </div>
    </div>
  )
}
