import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { format, parseISO } from 'date-fns'
import { api } from '../api/client'
import type { Activity } from '../types'
import { ActivityBadge } from '../components/ActivityBadge'
import { EmptyState } from '../components/EmptyState'

function fmtDist(m: number | null) {
  if (!m) return '—'
  return (m / 1000).toFixed(2) + ' km'
}

function fmtDuration(secs: number | null) {
  if (!secs) return '—'
  const h = Math.floor(secs / 3600)
  const m = Math.floor((secs % 3600) / 60)
  return h > 0 ? `${h}h ${m}m` : `${m}m`
}

function fmtPace(speed: number | null) {
  if (!speed || speed === 0) return '—'
  const minPerKm = 1000 / 60 / speed
  const min = Math.floor(minPerKm)
  const sec = Math.round((minPerKm - min) * 60)
  return `${min}:${sec.toString().padStart(2, '0')} /km`
}

function fmtDate(iso: string) {
  try {
    return format(parseISO(iso.replace(' ', 'T')), 'MMM d, yyyy')
  } catch {
    return iso.slice(0, 10)
  }
}

const ACTIVITY_TYPES = [
  'All', 'Running', 'Cycling', 'Swimming', 'Strength', 'Hiking', 'Walking',
]

export function Activities() {
  const [activities, setActivities] = useState<Activity[]>([])
  const [loading, setLoading] = useState(true)
  const [typeFilter, setTypeFilter] = useState('All')
  const [search, setSearch] = useState('')
  const navigate = useNavigate()

  useEffect(() => {
    setLoading(true)
    api
      .activities({
        activity_type: typeFilter !== 'All' ? typeFilter : undefined,
        limit: 200,
      })
      .then(setActivities)
      .finally(() => setLoading(false))
  }, [typeFilter])

  const filtered = search
    ? activities.filter(
        (a) =>
          (a.name ?? '').toLowerCase().includes(search.toLowerCase()) ||
          (a.activity_type ?? '').toLowerCase().includes(search.toLowerCase()),
      )
    : activities

  return (
    <div className="p-6 space-y-4">
      <div>
        <h1 className="text-2xl font-bold text-slate-100">Activities</h1>
        <p className="text-slate-400 text-sm mt-1">{activities.length} activities</p>
      </div>

      <div className="flex flex-wrap gap-3 items-center">
        <input
          type="text"
          placeholder="Search activities..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="bg-slate-800 border border-slate-600 rounded-lg px-3 py-2 text-sm text-slate-200 placeholder-slate-500 focus:outline-none focus:border-indigo-500 w-56"
        />
        <div className="flex gap-1 flex-wrap">
          {ACTIVITY_TYPES.map((t) => (
            <button
              key={t}
              onClick={() => setTypeFilter(t)}
              className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
                typeFilter === t
                  ? 'bg-indigo-600 text-white'
                  : 'bg-slate-800 text-slate-400 hover:text-slate-200 border border-slate-700'
              }`}
            >
              {t}
            </button>
          ))}
        </div>
      </div>

      {loading ? (
        <div className="text-slate-400 animate-pulse py-12 text-center">Loading...</div>
      ) : filtered.length === 0 ? (
        <EmptyState
          title="No activities found"
          description="Import your Garmin export to see your activities here."
          command="garmin-dash import /path/to/DI_CONNECT_export/"
        />
      ) : (
        <div className="bg-slate-800 rounded-xl overflow-hidden">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-slate-700 text-slate-400 text-xs uppercase tracking-wide">
                <th className="text-left px-4 py-3">Date</th>
                <th className="text-left px-4 py-3">Name</th>
                <th className="text-left px-4 py-3">Type</th>
                <th className="text-right px-4 py-3">Distance</th>
                <th className="text-right px-4 py-3">Duration</th>
                <th className="text-right px-4 py-3">Pace</th>
                <th className="text-right px-4 py-3">HR</th>
                <th className="text-right px-4 py-3">Calories</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-700/50">
              {filtered.map((act) => (
                <tr
                  key={act.id}
                  onClick={() => navigate(`/activities/${act.id}`)}
                  className="hover:bg-slate-750 cursor-pointer transition-colors hover:bg-slate-700/50"
                >
                  <td className="px-4 py-3 text-slate-400 whitespace-nowrap">
                    {fmtDate(act.start_time)}
                  </td>
                  <td className="px-4 py-3 text-slate-200 font-medium max-w-[180px] truncate">
                    {act.name ?? '—'}
                  </td>
                  <td className="px-4 py-3">
                    <ActivityBadge type={act.activity_type} />
                  </td>
                  <td className="px-4 py-3 text-right text-blue-400">
                    {fmtDist(act.distance_meters)}
                  </td>
                  <td className="px-4 py-3 text-right text-slate-300">
                    {fmtDuration(act.duration_secs)}
                  </td>
                  <td className="px-4 py-3 text-right text-slate-400">
                    {fmtPace(act.avg_speed)}
                  </td>
                  <td className="px-4 py-3 text-right text-rose-400">
                    {act.avg_hr ? `${act.avg_hr} bpm` : '—'}
                  </td>
                  <td className="px-4 py-3 text-right text-orange-400">
                    {act.calories ? `${act.calories} kcal` : '—'}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  )
}
