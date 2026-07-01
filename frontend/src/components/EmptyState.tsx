interface EmptyStateProps {
  title: string
  description: string
  command?: string
}

export function EmptyState({ title, description, command }: EmptyStateProps) {
  return (
    <div className="flex flex-col items-center justify-center py-24 text-center">
      <div className="text-5xl mb-4">📂</div>
      <h3 className="text-xl font-semibold text-slate-200 mb-2">{title}</h3>
      <p className="text-slate-400 max-w-sm mb-6">{description}</p>
      {command && (
        <div className="bg-slate-800 border border-slate-600 rounded-lg px-5 py-3 font-mono text-sm text-emerald-400">
          {command}
        </div>
      )}
    </div>
  )
}
