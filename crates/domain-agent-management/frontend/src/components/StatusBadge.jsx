// crates/agent-management/frontend/src/components/StatusBadge.jsx
import { getStatusBadgeClass } from '../utils/formatters'

export function StatusBadge({ status }) {
  const className = getStatusBadgeClass(status)
  return (
    <span className={`px-2 py-1 text-xs rounded-full ${className}`}>
      {status}
    </span>
  )
}