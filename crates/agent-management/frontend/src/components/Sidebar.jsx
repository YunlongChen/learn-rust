// crates/agent-management/frontend/src/components/Sidebar.jsx
import { NavLink } from 'react-router-dom'

const navItems = [
  { to: '/', label: 'Dashboard', icon: '📊' },
  { to: '/agents', label: 'Agents', icon: '🤖' },
  { to: '/events', label: 'Events', icon: '📋' },
  { to: '/health', label: 'Health', icon: '❤️' },
  { to: '/settings', label: 'Settings', icon: '⚙️' }
]

export function Sidebar() {
  return (
    <aside className="w-60 bg-white border-r border-gray-200 py-5">
      <div className="px-6 pb-5 border-b border-gray-200 mb-5">
        <h1 className="text-lg font-semibold text-gray-800">AgentHub</h1>
        <span className="text-xs text-gray-400">Management Console</span>
      </div>
      <nav>
        {navItems.map(({ to, label, icon }) => (
          <NavLink
            key={to}
            to={to}
            className={({ isActive }) =>
              `px-6 py-3 cursor-pointer flex items-center gap-3 transition ${
                isActive
                  ? 'bg-blue-50 text-primary border-r-4 border-primary'
                  : 'text-gray-600 hover:bg-gray-50 hover:text-primary'
              }`
            }
          >
            <span>{icon}</span>
            <span>{label}</span>
          </NavLink>
        ))}
      </nav>
    </aside>
  )
}