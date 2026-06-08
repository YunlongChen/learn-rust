// crates/agent-management/frontend/src/utils/formatters.js
export const formatDate = (dateStr) => {
  if (!dateStr) return 'N/A'
  return new Date(dateStr).toLocaleString()
}

export const getHealthStatus = (score) => {
  if (score >= 90) return { label: 'Excellent', class: 'text-success' }
  if (score >= 70) return { label: 'Good', class: 'text-primary' }
  if (score >= 50) return { label: 'Fair', class: 'text-warning' }
  if (score >= 30) return { label: 'Poor', class: 'text-danger' }
  return { label: 'Critical', class: 'text-danger' }
}

export const getStatusBadgeClass = (status) => {
  const classes = {
    created: 'bg-gray-100 text-gray-600',
    pending: 'bg-yellow-100 text-yellow-600',
    authorized: 'bg-blue-100 text-blue-600',
    connected: 'bg-blue-100 text-blue-700',
    registered: 'bg-green-100 text-green-600',
    reconnecting: 'bg-yellow-50 text-yellow-700',
    closed: 'bg-gray-100 text-gray-500'
  }
  return classes[status] || classes.created
}
