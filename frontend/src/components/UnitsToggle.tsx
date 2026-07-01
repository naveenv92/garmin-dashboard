import { useUnits } from '../context/UnitsContext'

export function UnitsToggle() {
  const { units, setUnits } = useUnits()

  return (
    <div className="inline-flex rounded-lg bg-slate-800 border border-slate-700 p-0.5 text-xs font-medium">
      {(['metric', 'imperial'] as const).map((u) => (
        <button
          key={u}
          onClick={() => setUnits(u)}
          className={`px-3 py-1.5 rounded-md capitalize transition-colors ${
            units === u
              ? 'bg-indigo-600 text-white'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          {u}
        </button>
      ))}
    </div>
  )
}
