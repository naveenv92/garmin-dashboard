import { useEffect, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import {
  LineChart, Line, XAxis, YAxis, Tooltip,
  ResponsiveContainer, CartesianGrid, AreaChart, Area,
} from 'recharts'
import { format, parseISO } from 'date-fns'
import { api } from '../api/client'
import type { ActivityWithRecords } from '../types'
import { ActivityBadge } from '../components/ActivityBadge'
import { ActivityMap } from '../components/ActivityMap'
import { useUnits } from '../context/UnitsContext'

function fmtDuration(secs: number | null) {
  if (!secs) return '—'
  const h = Math.floor(secs / 3600)
  const m = Math.floor((secs % 3600) / 60)
  const s = Math.round(secs % 60)
  return h > 0 ? `${h}h ${m}m ${s}s` : `${m}m ${s}s`
}

function fmtDate(iso: string) {
  try {
    return format(parseISO(iso.replace(' ', 'T')), 'EEEE, MMMM d, yyyy · HH:mm')
  } catch {
    return iso
  }
}

const CustomTooltip = ({ active, payload, label }: any) => {
  if (!active || !payload?.length) return null
  return (
    <div className="bg-slate-800 border border-slate-600 rounded-lg px-3 py-2 text-xs">
      <div className="text-slate-400 mb-1">{label}</div>
      {payload.map((p: any, i: number) => (
        <div key={i} style={{ color: p.color }}>
          {p.name}: {p.value}
        </div>
      ))}
    </div>
  )
}

interface StatRowProps {
  label: string
  value: string
  color?: string
}

function StatRow({ label, value, color }: StatRowProps) {
  return (
    <div className="flex justify-between items-center py-2 border-b border-slate-700/50">
      <span className="text-slate-400 text-sm">{label}</span>
      <span className={`text-sm font-semibold ${color ?? 'text-slate-200'}`}>{value}</span>
    </div>
  )
}

const METERS_PER_FOOT = 0.3048

export function ActivityDetail() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const { units, elevationUnit, formatDistance, formatElevation, formatPace, formatSpeed } =
    useUnits()
  const [data, setData] = useState<ActivityWithRecords | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState(false)

  useEffect(() => {
    if (!id) return
    api
      .activity(Number(id))
      .then(setData)
      .catch(() => setError(true))
      .finally(() => setLoading(false))
  }, [id])

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-slate-400 animate-pulse">Loading...</div>
      </div>
    )
  }

  if (error || !data) {
    return (
      <div className="p-6 text-center">
        <div className="text-slate-400">Activity not found.</div>
        <button onClick={() => navigate('/activities')} className="mt-4 text-indigo-400 hover:underline text-sm">
          ← Back to activities
        </button>
      </div>
    )
  }

  const { activity, records, laps } = data
  const lapLabel = activity.activity_type?.toLowerCase().includes('ski') ? 'Run' : 'Lap'

  // Downsample records for chart performance (max 500 points)
  const step = Math.max(1, Math.floor(records.length / 500))
  const sampledRecords = records.filter((_, i) => i % step === 0)

  const chartData = sampledRecords.map((r, i) => ({
    i,
    time: r.timestamp ? format(parseISO(r.timestamp), 'HH:mm:ss') : String(i),
    dist: r.distance ? (r.distance / 1000).toFixed(2) : null,
    hr: r.heart_rate,
    alt: r.altitude
      ? Math.round(units === 'imperial' ? r.altitude / METERS_PER_FOOT : r.altitude)
      : null,
    speed: r.speed
      ? Math.round((1000 / 60 / r.speed) * 10) / 10
      : null,
    cadence: r.cadence,
    power: r.power,
  }))

  const hasGps = records.some((r) => r.lat !== null && r.lon !== null)
  const hasHr = records.some((r) => r.heart_rate !== null)
  const hasAlt = records.some((r) => r.altitude !== null)
  const hasPower = records.some((r) => r.power !== null)

  return (
    <div className="p-6 space-y-6 max-w-5xl">
      <div className="flex items-start gap-4">
        <button
          onClick={() => navigate('/activities')}
          className="text-slate-400 hover:text-slate-200 text-sm mt-1"
        >
          ←
        </button>
        <div>
          <div className="flex items-center gap-3 mb-1">
            <ActivityBadge type={activity.activity_type} />
            {activity.vo2max && (
              <span className="text-xs text-slate-500">VO₂max {activity.vo2max}</span>
            )}
          </div>
          <h1 className="text-2xl font-bold text-slate-100">
            {activity.name ?? activity.activity_type ?? 'Activity'}
          </h1>
          <p className="text-slate-400 text-sm mt-0.5">{fmtDate(activity.start_time)}</p>
        </div>
      </div>

      <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
        {[
          { label: 'Distance', value: formatDistance(activity.distance_meters), color: 'text-blue-400' },
          { label: 'Duration', value: fmtDuration(activity.duration_secs), color: 'text-slate-200' },
          { label: 'Avg Pace', value: formatPace(activity.avg_speed), color: 'text-emerald-400' },
          { label: 'Calories', value: activity.calories ? `${activity.calories} kcal` : '—', color: 'text-orange-400' },
        ].map((s) => (
          <div key={s.label} className="bg-slate-800 rounded-xl p-4">
            <div className="text-xs text-slate-500 uppercase tracking-wide mb-1">{s.label}</div>
            <div className={`text-xl font-bold ${s.color}`}>{s.value}</div>
          </div>
        ))}
      </div>

      {hasGps && <ActivityMap records={records} />}

      <div className="grid grid-cols-1 xl:grid-cols-3 gap-4">
        <div className="bg-slate-800 rounded-xl p-4 space-y-0">
          <h2 className="text-xs font-semibold text-slate-400 uppercase tracking-wide mb-3">Stats</h2>
          <StatRow label="Avg Heart Rate" value={activity.avg_hr ? `${activity.avg_hr} bpm` : '—'} color="text-rose-400" />
          <StatRow label="Max Heart Rate" value={activity.max_hr ? `${activity.max_hr} bpm` : '—'} color="text-rose-400" />
          <StatRow label="Max Speed" value={formatSpeed(activity.max_speed)} color="text-cyan-400" />
          <StatRow label="Elevation Gain" value={formatElevation(activity.elevation_gain)} />
          <StatRow label="Elevation Loss" value={formatElevation(activity.elevation_loss)} />
          <StatRow label="Avg Cadence" value={activity.avg_cadence ? `${activity.avg_cadence} rpm` : '—'} />
          {activity.avg_power && (
            <StatRow label="Avg Power" value={`${activity.avg_power} W`} color="text-yellow-400" />
          )}
          <StatRow label="Track Points" value={records.length.toLocaleString()} />
          {hasGps ? (
            <StatRow label="GPS" value="✓ Available" color="text-emerald-400" />
          ) : (
            <StatRow label="GPS" value="Not available" color="text-slate-500" />
          )}
        </div>

        <div className="xl:col-span-2 space-y-4">
          {hasHr && (
            <div className="bg-slate-800 rounded-xl p-4">
              <h2 className="text-xs font-semibold text-slate-400 uppercase tracking-wide mb-3">
                Heart Rate
              </h2>
              <ResponsiveContainer width="100%" height={150}>
                <AreaChart data={chartData} margin={{ top: 4, right: 4, left: 0, bottom: 4 }}>
                  <defs>
                    <linearGradient id="hrGrad" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="5%" stopColor="#f43f5e" stopOpacity={0.3} />
                      <stop offset="95%" stopColor="#f43f5e" stopOpacity={0} />
                    </linearGradient>
                  </defs>
                  <CartesianGrid strokeDasharray="3 3" stroke="#1e293b" />
                  <XAxis
                    dataKey="time"
                    tick={{ fontSize: 10, fill: '#64748b' }}
                    minTickGap={40}
                  />
                  <YAxis tick={{ fontSize: 10, fill: '#64748b' }} unit=" bpm" domain={['auto', 'auto']} width={58} />
                  <Tooltip content={<CustomTooltip />} />
                  <Area dataKey="hr" name="HR" stroke="#f43f5e" fill="url(#hrGrad)" strokeWidth={1.5} dot={false} />
                </AreaChart>
              </ResponsiveContainer>
            </div>
          )}

          {hasAlt && (
            <div className="bg-slate-800 rounded-xl p-4">
              <h2 className="text-xs font-semibold text-slate-400 uppercase tracking-wide mb-3">
                Elevation
              </h2>
              <ResponsiveContainer width="100%" height={120}>
                <AreaChart data={chartData} margin={{ top: 4, right: 4, left: 0, bottom: 4 }}>
                  <defs>
                    <linearGradient id="altGrad" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="5%" stopColor="#22d3ee" stopOpacity={0.3} />
                      <stop offset="95%" stopColor="#22d3ee" stopOpacity={0} />
                    </linearGradient>
                  </defs>
                  <CartesianGrid strokeDasharray="3 3" stroke="#1e293b" />
                  <XAxis
                    dataKey="time"
                    tick={{ fontSize: 10, fill: '#64748b' }}
                    minTickGap={40}
                  />
                  <YAxis tick={{ fontSize: 10, fill: '#64748b' }} unit={` ${elevationUnit}`} domain={['auto', 'auto']} width={58} />
                  <Tooltip content={<CustomTooltip />} />
                  <Area dataKey="alt" name="Altitude" stroke="#22d3ee" fill="url(#altGrad)" strokeWidth={1.5} dot={false} />
                </AreaChart>
              </ResponsiveContainer>
            </div>
          )}

          {hasPower && (
            <div className="bg-slate-800 rounded-xl p-4">
              <h2 className="text-xs font-semibold text-slate-400 uppercase tracking-wide mb-3">
                Power
              </h2>
              <ResponsiveContainer width="100%" height={120}>
                <LineChart data={chartData} margin={{ top: 4, right: 4, left: 0, bottom: 4 }}>
                  <CartesianGrid strokeDasharray="3 3" stroke="#1e293b" />
                  <XAxis
                    dataKey="time"
                    tick={{ fontSize: 10, fill: '#64748b' }}
                    minTickGap={40}
                  />
                  <YAxis tick={{ fontSize: 10, fill: '#64748b' }} unit=" W" domain={['auto', 'auto']} width={44} />
                  <Tooltip content={<CustomTooltip />} />
                  <Line dataKey="power" name="Power" stroke="#facc15" strokeWidth={1.5} dot={false} />
                </LineChart>
              </ResponsiveContainer>
            </div>
          )}
        </div>
      </div>

      {laps.length > 1 && (
        <div className="bg-slate-800 rounded-xl p-4">
          <h2 className="text-xs font-semibold text-slate-400 uppercase tracking-wide mb-3">
            {lapLabel}s ({laps.length})
          </h2>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-slate-700 text-slate-400 text-xs uppercase tracking-wide">
                  <th className="text-left px-3 py-2">#</th>
                  <th className="text-right px-3 py-2">Duration</th>
                  <th className="text-right px-3 py-2">Distance</th>
                  <th className="text-right px-3 py-2">Descent</th>
                  <th className="text-right px-3 py-2">Max Speed</th>
                  <th className="text-right px-3 py-2">Avg HR</th>
                  <th className="text-right px-3 py-2">Max HR</th>
                  <th className="text-right px-3 py-2">Calories</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-700/50">
                {laps.map((lap) => (
                  <tr key={lap.id}>
                    <td className="px-3 py-2 text-slate-300">{lap.lap_index + 1}</td>
                    <td className="px-3 py-2 text-right text-slate-300">{fmtDuration(lap.duration_secs)}</td>
                    <td className="px-3 py-2 text-right text-blue-400">{formatDistance(lap.distance_meters)}</td>
                    <td className="px-3 py-2 text-right text-slate-300">{formatElevation(lap.elevation_loss)}</td>
                    <td className="px-3 py-2 text-right text-cyan-400">{formatSpeed(lap.max_speed)}</td>
                    <td className="px-3 py-2 text-right text-rose-400">{lap.avg_hr ? `${lap.avg_hr} bpm` : '—'}</td>
                    <td className="px-3 py-2 text-right text-rose-400">{lap.max_hr ? `${lap.max_hr} bpm` : '—'}</td>
                    <td className="px-3 py-2 text-right text-orange-400">{lap.calories ? `${lap.calories} kcal` : '—'}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {!hasGps && records.length === 0 && (
        <div className="bg-slate-800 rounded-xl p-6 text-center text-slate-500 text-sm">
          No per-second track data available. Import the corresponding FIT or GPX file to see charts.
        </div>
      )}
    </div>
  )
}
