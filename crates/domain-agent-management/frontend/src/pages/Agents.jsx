// crates/agent-management/frontend/src/pages/Agents.jsx
import { useState } from 'react'
import { useAgents } from '../hooks/useAgents'
import { AgentTable } from '../components/AgentTable'
import { CreateAgentModal } from '../components/CreateAgentModal'
import { ApproveModal } from '../components/ApproveModal'
import { DenyModal } from '../components/DenyModal'
import { DeleteModal } from '../components/DeleteModal'
import { Header } from '../components/Header'

export function Agents() {
  const { agents, loading, createAgent, approveAgent, denyAgent, deleteAgent } = useAgents()
  const [showCreate, setShowCreate] = useState(false)
  const [approveTarget, setApproveTarget] = useState(null)
  const [denyTarget, setDenyTarget] = useState(null)
  const [deleteTarget, setDeleteTarget] = useState(null)

  return (
    <>
      <Header
        title="Agents"
        actions={<button onClick={() => setShowCreate(true)} className="px-4 py-2 bg-primary text-white rounded hover:bg-blue-400 text-sm font-medium">+ New Agent</button>}
      />
      <div className="flex-1 p-6">
        <div className="bg-white rounded-lg border border-gray-200">
          {loading ? (
            <div className="p-6 text-center text-gray-500">Loading...</div>
          ) : (
            <AgentTable
              agents={agents}
              onApprove={(agent) => setApproveTarget(agent)}
              onDeny={(agent) => setDenyTarget(agent)}
              onDelete={(agent) => setDeleteTarget(agent)}
            />
          )}
        </div>
      </div>
      <CreateAgentModal isOpen={showCreate} onClose={() => setShowCreate(false)} onSubmit={createAgent} />
      <ApproveModal isOpen={!!approveTarget} agent={approveTarget} onClose={() => setApproveTarget(null)} onApprove={approveAgent} />
      <DenyModal isOpen={!!denyTarget} agent={denyTarget} onClose={() => setDenyTarget(null)} onDeny={denyAgent} />
      <DeleteModal isOpen={!!deleteTarget} agent={deleteTarget} onClose={() => setDeleteTarget(null)} onDelete={deleteAgent} />
    </>
  )
}
