// crates/agent-management/frontend/src/api/agent.js
import axios from 'axios'

const API_BASE = '/api/v1'

const api = axios.create({
  baseURL: API_BASE,
  headers: { 'Content-Type': 'application/json' }
})

export const agentApi = {
  list: () => api.get('/agents'),
  get: (id) => api.get(`/agents/${id}`),
  create: (data) => api.post('/agents', data),
  update: (id, data) => api.patch(`/agents/${id}`, data),
  delete: (id) => api.delete(`/agents/${id}`),
  approve: (id) => api.post(`/agents/${id}/approve`),
  deny: (id, reason) => api.post(`/agents/${id}/deny`, { reason }),
  getSystemInfo: (id) => api.get(`/agents/${id}/system-info`),
  getHealth: (id) => api.get(`/agents/${id}/health`),
  getLifecycle: (id) => api.get(`/agents/${id}/lifecycle`)
}

export default agentApi
