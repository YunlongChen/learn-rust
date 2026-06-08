// crates/agent-management/frontend/src/components/PageContainer.jsx
export function PageContainer({ title, actions, children }) {
  return (
    <div className="flex-1 p-6">
      <div className="bg-white rounded-lg border border-gray-200">
        <div className="px-5 py-4 border-b border-gray-200 flex justify-between items-center">
          <span className="font-semibold text-gray-800">{title}</span>
          {actions}
        </div>
        {children}
      </div>
    </div>
  )
}