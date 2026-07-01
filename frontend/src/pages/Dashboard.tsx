import { useEffect, useState } from 'react'
import {
  BarChart, Bar, LineChart, Line, XAxis, YAxis,
  Tooltip, ResponsiveContainer, CartesianGrid,
} from 'recharts'
import { api } from '../api/client'
import type { DailyWellness, OverviewStats } from '../types'
import { StatCard } from '../components/StatCard'
import { ActivityBadge } from '../components/ActivityBadge'
import { EmptyState } from '../components/EmptyState'
import { useNavigate } from 'react-router-dom'
import { format, parseISO } from 'date-fns'
import { useUnits } from '../context/UnitsContext'

function fmtDuration(secs: number | null) {
  if (!secs) return '—'
  const h = Math.floor(secs / 3600)
  const m = Math.floor((secs % 3600) / 60)
  return h > 0 ? `${h}h ${m}m` : `${m}m`
}

function fmtDate(iso: string) {
  try {
    return format(parseISO(iso.replace(' ', 'T')), 'MMM d, yyyy')
  } catch {
    return iso.slice(0, 10)
  }
}

const CustomTooltip = ({ active, payload, label }: any) => {
  if (!active || !payload?.length) return null
  return (
    <div className="bg-slate-800 border border-slate-600 rounded-lg px-3 py-2 text-sm">
      <div className="text-slate-400 mb-1">{label}</div>
      {payload.map((p: any, i: number) => (
        <div key={i} style={{ color: p.color }}>{p.name}: {p.value?.toLocaleString()}</div>
      ))}
    </div>
  )
}

export function Dashboard() {
  const { formatDistance, formatDistanceKm } = useUnits()
  const [stats, setStats] = useState<OverviewStats | null>(null)
  const [wellness, setWellness] = useState<DailyWellness[]>([])
  const [error, setError] = useState(false)
  const navigate = useNavigate()

  useEffect(() => {
    api.overview().then(setStats).catch(() => setError(true))
    api.wellnessDaily({ limit: 30 }).then(setWellness).catch(() => {})
  }, [])

  if (error) {
    return (
      <EmptyState
        title="No data yet"
        description="Import your Garmin Connect export to get started."
        command="garmin-dash import /path/to/DI_CONNECT_export/"
      />
    )
  }

  if (!stats) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-slate-400 animate-pulse">Loading...</div>
      </div>
    )
  }

  const stepsData = [...wellness]
    .reverse()
    .map((d) => ({
      date: d.date,
      steps: d.steps ?? 0,
    }))

  const sleepData = [...wellness]
    .reverse()
    .filter((d) => d.sleep_seconds !== null)
    .map((d) => ({
      date: d.date.slice(5),
      hours: Math.round(((d.sleep_seconds ?? 0) / 3600) * 10) / 10,
    }))

  const isEmpty = stats.total_activities === 0 && stats.month_steps === 0

  if (isEmpty) {
    return (
      <EmptyState
        title="No data yet"
        description="Import your Garmin Connect export to populate the dashboard."
        command="garmin-dash import /path/to/DI_CONNECT_export/"
      />
    )
  }

  return (
    <div className="p-6 space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-slate-100">Dashboard</h1>
        <p className="text-slate-400 text-sm mt-1">Your Garmin fitness overview</p>
      </div>

      <div className="grid grid-cols-2 xl:grid-cols-4 gap-4">
        <StatCard
          label="Total Activities"
          value={stats.total_activities.toLocaleString()}
          color="indigo"
        />
        <StatCard
          label="Total Distance"
          value={formatDistanceKm(stats.total_distance_km)}
          color="blue"
        />
        <StatCard
          label="This Month Steps"
          value={stats.month_steps.toLocaleString()}
          color="green"
        />
        <StatCard
          label="Avg Resting HR"
          value={stats.avg_resting_hr ? Math.round(stats.avg_resting_hr) + ' bpm' : '—'}
          sub="last 30 days"
          color="rose"
        />
      </div>

      <div className="grid grid-cols-1 xl:grid-cols-2 gap-4">
        <div className="bg-slate-800 rounded-xl p-5">
          <h2 className="text-sm font-semibold text-slate-300 mb-4 uppercase tracking-wide">
            Steps — Last 30 Days
          </h2>
          {stepsData.length === 0 ? (
            <div className="text-slate-500 text-sm text-center py-8">No step data</div>
          ) : (
            <ResponsiveContainer width="100%" height={200}>
              <BarChart data={stepsData} margin={{ top: 4, right: 4, left: -10, bottom: 0 }}>
                <CartesianGrid strokeDasharray="3 3" stroke="#1e293b" />
                <XAxis dataKey="date" tick={{ fontSize: 11, fill: '#64748b' }} interval={6} />
                <YAxis tick={{ fontSize: 11, fill: '#64748b' }} />
                <Tooltip content={<CustomTooltip />} />
                <Bar dataKey="steps" name="Steps" fill="#10b981" radius={[3, 3, 0, 0]} />
              </BarChart>
            </ResponsiveContainer>
          )}
        </div>

        <div className="bg-slate-800 rounded-xl p-5">
          <h2 className="text-sm font-semibold text-slate-300 mb-4 uppercase tracking-wide">
            Sleep — Last 30 Days
          </h2>
          {sleepData.length === 0 ? (
            <div className="text-slate-500 text-sm text-center py-8">No sleep data</div>
          ) : (
            <ResponsiveContainer width="100%" height={200}>
              <LineChart data={sleepData} margin={{ top: 4, right: 4, left: -10, bottom: 0 }}>
                <CartesianGrid strokeDasharray="3 3" stroke="#1e293b" />
                <XAxis dataKey="date" tick={{ fontSize: 11, fill: '#64748b' }} interval={6} />
                <YAxis tick={{ fontSize: 11, fill: '#64748b' }} domain={[0, 12]} unit="h" />
                <Tooltip content={<CustomTooltip />} />
                <Line
                  dataKey="hours"
                  name="Sleep (hrs)"
                  stroke="#818cf8"
                  strokeWidth={2}
                  dot={false}
                />
              </LineChart>
            </ResponsiveContainer>
          )}
        </div>
      </div>

      <div className="bg-slate-800 rounded-xl p-5">
        <h2 className="text-sm font-semibold text-slate-300 mb-4 uppercase tracking-wide">
          Recent Activities
        </h2>
        {stats.recent_activities.length === 0 ? (
          <div className="text-slate-500 text-sm">No activities yet.</div>
        ) : (
          <div className="divide-y divide-slate-700">
            {stats.recent_activities.map((act) => (
              <button
                key={act.id}
                onClick={() => navigate(`/activities/${act.id}`)}
                className="w-full flex items-center gap-4 py-3 text-left hover:bg-slate-750 rounded-lg px-2 -mx-2 transition-colors group"
              >
                <ActivityBadge type={act.activity_type} />
                <div className="flex-1 min-w-0">
                  <div className="text-slate-200 text-sm font-medium truncate group-hover:text-white">
                    {act.name ?? act.activity_type ?? 'Activity'}
                  </div>
                  <div className="text-slate-500 text-xs">{fmtDate(act.start_time)}</div>
                </div>
                <div className="text-right text-sm text-slate-400 flex gap-4">
                  {act.distance_meters && (
                    <span className="text-blue-400">{formatDistance(act.distance_meters)}</span>
                  )}
                  {act.duration_secs && (
                    <span>{fmtDuration(act.duration_secs)}</span>
                  )}
                  {act.avg_hr && (
                    <span className="text-rose-400">{act.avg_hr} bpm</span>
                  )}
                </div>
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}
