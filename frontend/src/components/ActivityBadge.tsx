interface ActivityBadgeProps {
  type: string | null
}

function colorForType(type: string | null): string {
  const t = (type ?? '').toLowerCase()
  if (t.includes('run')) return 'bg-emerald-900 text-emerald-300'
  if (t.includes('cycl') || t.includes('bike') || t.includes('ride'))
    return 'bg-blue-900 text-blue-300'
  if (t.includes('swim')) return 'bg-cyan-900 text-cyan-300'
  if (t.includes('strength') || t.includes('gym') || t.includes('weight'))
    return 'bg-purple-900 text-purple-300'
  if (t.includes('hik') || t.includes('walk')) return 'bg-amber-900 text-amber-300'
  if (t.includes('yoga') || t.includes('pilates')) return 'bg-pink-900 text-pink-300'
  return 'bg-slate-700 text-slate-300'
}

export function ActivityBadge({ type }: ActivityBadgeProps) {
  return (
    <span className={`px-2 py-0.5 rounded text-xs font-medium ${colorForType(type)}`}>
      {type ?? 'Activity'}
    </span>
  )
}
