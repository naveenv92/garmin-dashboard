interface StatCardProps {
  label: string
  value: string | number
  sub?: string
  color: 'blue' | 'green' | 'orange' | 'rose' | 'purple' | 'indigo'
}

const colorMap = {
  blue: 'border-blue-500 text-blue-400',
  green: 'border-emerald-500 text-emerald-400',
  orange: 'border-orange-500 text-orange-400',
  rose: 'border-rose-500 text-rose-400',
  purple: 'border-purple-500 text-purple-400',
  indigo: 'border-indigo-500 text-indigo-400',
}

export function StatCard({ label, value, sub, color }: StatCardProps) {
  const colors = colorMap[color]
  return (
    <div className={`bg-slate-800 rounded-xl p-5 border-l-4 ${colors.split(' ')[0]}`}>
      <div className={`text-xs font-semibold uppercase tracking-wider mb-2 ${colors.split(' ')[1]}`}>
        {label}
      </div>
      <div className="text-3xl font-bold text-slate-100">{value}</div>
      {sub && <div className="text-slate-500 text-sm mt-1">{sub}</div>}
    </div>
  )
}
