import { useEffect, useState } from 'react'
import {
  BarChart, Bar, LineChart, Line, XAxis, YAxis,
  Tooltip, ResponsiveContainer, CartesianGrid, Legend,
  AreaChart, Area,
} from 'recharts'
import { api } from '../api/client'
import type { DailyWellness, SleepSession } from '../types'
import { EmptyState } from '../components/EmptyState'
import { StatCard } from '../components/StatCard'

type Range = '7d' | '30d' | '90d'

const RANGES: { label: string; value: Range; limit: number }[] = [
  { label: '7 days', value: '7d', limit: 7 },
  { label: '30 days', value: '30d', limit: 30 },
  { label: '90 days', value: '90d', limit: 90 },
]

const CustomTooltip = ({ active, payload, label }: any) => {
  if (!active || !payload?.length) return null
  return (
    <div className="bg-slate-800 border border-slate-600 rounded-lg px-3 py-2 text-xs">
      <div className="text-slate-400 mb-1">{label}</div>
      {payload.map((p: any, i: number) => (
        <div key={i} style={{ color: p.color }}>
          {p.name}: {typeof p.value === 'number' ? p.value.toLocaleString() : p.value}
        </div>
      ))}
    </div>
  )
}

export function Wellness() {
  const [wellness, setWellness] = useState<DailyWellness[]>([])
  const [sleep, setSleep] = useState<SleepSession[]>([])
  const [range, setRange] = useState<Range>('30d')
  const [loading, setLoading] = useState(true)

  const limit = RANGES.find((r) => r.value === range)!.limit

  useEffect(() => {
    setLoading(true)
    Promise.all([
      api.wellnessDaily({ limit }),
      api.wellnessSleep({ limit }),
    ])
      .then(([w, s]) => {
        setWellness(w)
        setSleep(s)
      })
      .finally(() => setLoading(false))
  }, [limit])

  const stepsData = [...wellness].reverse().map((d) => ({
    date: d.date.slice(5),
    steps: d.steps ?? 0,
  }))

  const sleepData = [...sleep].reverse().map((s) => ({
    date: s.date.slice(5),
    deep: Math.round(((s.deep_sleep_secs ?? 0) / 3600) * 10) / 10,
    light: Math.round(((s.light_sleep_secs ?? 0) / 3600) * 10) / 10,
    rem: Math.round(((s.rem_sleep_secs ?? 0) / 3600) * 10) / 10,
    awake: Math.round(((s.awake_secs ?? 0) / 3600) * 10) / 10,
    score: s.sleep_score ?? null,
  }))

  const hrvData = [...wellness]
    .reverse()
    .filter((d) => d.hrv_night_avg !== null)
    .map((d) => ({
      date: d.date.slice(5),
      hrv: Math.round((d.hrv_night_avg ?? 0) * 10) / 10,
    }))

  const stressData = [...wellness]
    .reverse()
    .filter((d) => d.avg_stress !== null || d.body_battery_high !== null)
    .map((d) => ({
      date: d.date.slice(5),
      stress: d.avg_stress ?? null,
      battery_high: d.body_battery_high ?? null,
      battery_low: d.body_battery_low ?? null,
    }))

  const avgSteps =
    stepsData.length > 0
      ? Math.round(stepsData.reduce((s, d) => s + d.steps, 0) / stepsData.length)
      : null

  const avgSleep =
    sleepData.length > 0
      ? Math.round(
          (sleepData.reduce(
            (s, d) => s + d.deep + d.light + d.rem,
            0,
          ) /
            sleepData.length) *
            10,
        ) / 10
      : null

  const avgHrv =
    hrvData.length > 0
      ? Math.round((hrvData.reduce((s, d) => s + d.hrv, 0) / hrvData.length) * 10) / 10
      : null

  const isEmpty = wellness.length === 0 && sleep.length === 0

  if (!loading && isEmpty) {
    return (
      <EmptyState
        title="No wellness data"
        description="Import your Garmin export to see your health trends."
        command="garmin-dash import /path/to/DI_CONNECT_export/"
      />
    )
  }

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-start justify-between">
        <div>
          <h1 className="text-2xl font-bold text-slate-100">Wellness</h1>
          <p className="text-slate-400 text-sm mt-1">Health & recovery trends</p>
        </div>
        <div className="flex gap-1">
          {RANGES.map((r) => (
            <button
              key={r.value}
              onClick={() => setRange(r.value)}
              className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
                range === r.value
                  ? 'bg-indigo-600 text-white'
                  : 'bg-slate-800 text-slate-400 hover:text-slate-200 border border-slate-700'
              }`}
            >
              {r.label}
            </button>
          ))}
        </div>
      </div>

      {loading ? (
        <div className="text-slate-400 animate-pulse py-12 text-center">Loading...</div>
      ) : (
        <>
          <div className="grid grid-cols-3 gap-4">
            <StatCard
              label="Avg Daily Steps"
              value={avgSteps?.toLocaleString() ?? '—'}
              sub={`over ${limit} days`}
              color="green"
            />
            <StatCard
              label="Avg Sleep"
              value={avgSleep ? `${avgSleep}h` : '—'}
              sub="total per night"
              color="indigo"
            />
            <StatCard
              label="Avg HRV"
              value={avgHrv ? `${avgHrv} ms` : '—'}
              sub="night average"
              color="purple"
            />
          </div>

          <div className="bg-slate-800 rounded-xl p-5">
            <h2 className="text-sm font-semibold text-slate-300 mb-4 uppercase tracking-wide">
              Daily Steps
            </h2>
            {stepsData.length === 0 ? (
              <div className="text-slate-500 text-sm text-center py-8">No step data available</div>
            ) : (
              <ResponsiveContainer width="100%" height={200}>
                <BarChart data={stepsData} margin={{ top: 4, right: 4, left: -10, bottom: 0 }}>
                  <CartesianGrid strokeDasharray="3 3" stroke="#1e293b" />
                  <XAxis dataKey="date" tick={{ fontSize: 10, fill: '#64748b' }} interval={Math.floor(stepsData.length / 7)} />
                  <YAxis tick={{ fontSize: 10, fill: '#64748b' }} />
                  <Tooltip content={<CustomTooltip />} />
                  <Bar dataKey="steps" name="Steps" fill="#10b981" radius={[2, 2, 0, 0]} />
                </BarChart>
              </ResponsiveContainer>
            )}
          </div>

          <div className="bg-slate-800 rounded-xl p-5">
            <h2 className="text-sm font-semibold text-slate-300 mb-4 uppercase tracking-wide">
              Sleep Breakdown
            </h2>
            {sleepData.length === 0 ? (
              <div className="text-slate-500 text-sm text-center py-8">No sleep data available</div>
            ) : (
              <ResponsiveContainer width="100%" height={220}>
                <BarChart data={sleepData} margin={{ top: 4, right: 4, left: -10, bottom: 0 }}>
                  <CartesianGrid strokeDasharray="3 3" stroke="#1e293b" />
                  <XAxis dataKey="date" tick={{ fontSize: 10, fill: '#64748b' }} interval={Math.floor(sleepData.length / 7)} />
                  <YAxis tick={{ fontSize: 10, fill: '#64748b' }} unit="h" />
                  <Tooltip content={<CustomTooltip />} />
                  <Legend wrapperStyle={{ fontSize: 11, color: '#94a3b8' }} />
                  <Bar dataKey="deep" name="Deep" stackId="a" fill="#6366f1" />
                  <Bar dataKey="rem" name="REM" stackId="a" fill="#8b5cf6" />
                  <Bar dataKey="light" name="Light" stackId="a" fill="#818cf8" />
                  <Bar dataKey="awake" name="Awake" stackId="a" fill="#334155" radius={[2, 2, 0, 0]} />
                </BarChart>
              </ResponsiveContainer>
            )}
          </div>

          <div className="grid grid-cols-1 xl:grid-cols-2 gap-4">
            <div className="bg-slate-800 rounded-xl p-5">
              <h2 className="text-sm font-semibold text-slate-300 mb-4 uppercase tracking-wide">
                HRV (Heart Rate Variability)
              </h2>
              {hrvData.length === 0 ? (
                <div className="text-slate-500 text-sm text-center py-8">No HRV data available</div>
              ) : (
                <ResponsiveContainer width="100%" height={180}>
                  <AreaChart data={hrvData} margin={{ top: 4, right: 4, left: -10, bottom: 0 }}>
                    <defs>
                      <linearGradient id="hrvGrad" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="5%" stopColor="#a855f7" stopOpacity={0.3} />
                        <stop offset="95%" stopColor="#a855f7" stopOpacity={0} />
                      </linearGradient>
                    </defs>
                    <CartesianGrid strokeDasharray="3 3" stroke="#1e293b" />
                    <XAxis dataKey="date" tick={{ fontSize: 10, fill: '#64748b' }} interval={Math.floor(hrvData.length / 5)} />
                    <YAxis tick={{ fontSize: 10, fill: '#64748b' }} unit=" ms" />
                    <Tooltip content={<CustomTooltip />} />
                    <Area
                      dataKey="hrv"
                      name="HRV"
                      stroke="#a855f7"
                      fill="url(#hrvGrad)"
                      strokeWidth={2}
                      dot={false}
                    />
                  </AreaChart>
                </ResponsiveContainer>
              )}
            </div>

            <div className="bg-slate-800 rounded-xl p-5">
              <h2 className="text-sm font-semibold text-slate-300 mb-4 uppercase tracking-wide">
                Stress & Body Battery
              </h2>
              {stressData.length === 0 ? (
                <div className="text-slate-500 text-sm text-center py-8">No stress/battery data</div>
              ) : (
                <ResponsiveContainer width="100%" height={180}>
                  <LineChart data={stressData} margin={{ top: 4, right: 4, left: -10, bottom: 0 }}>
                    <CartesianGrid strokeDasharray="3 3" stroke="#1e293b" />
                    <XAxis dataKey="date" tick={{ fontSize: 10, fill: '#64748b' }} interval={Math.floor(stressData.length / 5)} />
                    <YAxis tick={{ fontSize: 10, fill: '#64748b' }} />
                    <Tooltip content={<CustomTooltip />} />
                    <Legend wrapperStyle={{ fontSize: 11, color: '#94a3b8' }} />
                    <Line dataKey="stress" name="Stress" stroke="#f97316" strokeWidth={2} dot={false} />
                    <Line dataKey="battery_high" name="Body Battery" stroke="#22d3ee" strokeWidth={2} dot={false} />
                  </LineChart>
                </ResponsiveContainer>
              )}
            </div>
          </div>
        </>
      )}
    </div>
  )
}
