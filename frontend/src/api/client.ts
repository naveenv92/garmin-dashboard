import type {
  Activity,
  ActivityWithRecords,
  DailyWellness,
  OverviewStats,
  SleepSession,
} from '../types'

const BASE = '/api'

async function get<T>(path: string): Promise<T> {
  const res = await fetch(BASE + path)
  if (!res.ok) throw new Error(`API error ${res.status}: ${path}`)
  return res.json()
}

export const api = {
  overview: (): Promise<OverviewStats> => get('/stats/overview'),

  activities: (params?: {
    activity_type?: string
    from?: string
    to?: string
    limit?: number
    offset?: number
  }): Promise<Activity[]> => {
    const q = new URLSearchParams()
    if (params?.activity_type) q.set('activity_type', params.activity_type)
    if (params?.from) q.set('from', params.from)
    if (params?.to) q.set('to', params.to)
    if (params?.limit) q.set('limit', String(params.limit))
    if (params?.offset) q.set('offset', String(params.offset))
    const qs = q.toString()
    return get(`/activities${qs ? '?' + qs : ''}`)
  },

  activity: (id: number): Promise<ActivityWithRecords> =>
    get(`/activities/${id}`),

  wellnessDaily: (params?: {
    from?: string
    to?: string
    limit?: number
  }): Promise<DailyWellness[]> => {
    const q = new URLSearchParams()
    if (params?.from) q.set('from', params.from)
    if (params?.to) q.set('to', params.to)
    if (params?.limit) q.set('limit', String(params.limit))
    const qs = q.toString()
    return get(`/wellness/daily${qs ? '?' + qs : ''}`)
  },

  wellnessSleep: (params?: {
    from?: string
    to?: string
    limit?: number
  }): Promise<SleepSession[]> => {
    const q = new URLSearchParams()
    if (params?.from) q.set('from', params.from)
    if (params?.to) q.set('to', params.to)
    if (params?.limit) q.set('limit', String(params.limit))
    const qs = q.toString()
    return get(`/wellness/sleep${qs ? '?' + qs : ''}`)
  },
}
