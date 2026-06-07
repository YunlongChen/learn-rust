// crates/agent-management/frontend/src/components/Header.jsx
import { Link } from 'react-router-dom'

export function Header({ title, actions, showBack, onBack }) {
  if (showBack) {
    return (
      <header className="bg-white px-6 py-4 border-b border-gray-200 flex justify-between items-center">
        <button onClick={onBack} className="flex items-center gap-2 text-gray-600 hover:text-primary transition">
          <span>←</span>
          <span>Back</span>
        </button>
        <h2 className="text-xl font-semibold text-gray-800">{title}</h2>
        <div className="flex gap-3">{actions}</div>
      </header>
    )
  }

  return (
    <header className="bg-white px-6 py-4 border-b border-gray-200 flex justify-between items-center">
      <h2 className="text-xl font-semibold text-gray-800">{title}</h2>
      {actions}
    </header>
  )
}