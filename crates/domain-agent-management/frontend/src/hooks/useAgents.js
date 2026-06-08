// crates/agent-management/frontend/src/hooks/useAgents.js
import { useState, useEffect, useCallback } from 'react'
import agentApi from '../api/agent'

export function useAgents() {
  const [agents, setAgents] = useState([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState(null)

  const fetchAgents = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const response = await agentApi.list()
      setAgents(response.data.agents || [])
    } catch (err) {
      setError(err.message)
    } finally {
      setLoading(false)
    }
  }, [])

  const createAgent = async (data) => {
    const response = await agentApi.create(data)
    await fetchAgents()
    return response.data
  }

  const updateAgent = async (id, data) => {
    const response = await agentApi.update(id, data)
    await fetchAgents()
    return response.data
  }

  const deleteAgent = async (id) => {
    await agentApi.delete(id)
    await fetchAgents()
  }

  const approveAgent = async (id) => {
    const response = await agentApi.approve(id)
    await fetchAgents()
    return response.data
  }

  const denyAgent = async (id, reason) => {
    const response = await agentApi.deny(id, reason)
    await fetchAgents()
    return response.data
  }

  useEffect(() => {
    fetchAgents()
  }, [fetchAgents])

  return {
    agents,
    loading,
    error,
    fetchAgents,
    createAgent,
    updateAgent,
    deleteAgent,
    approveAgent,
    denyAgent
  }
}