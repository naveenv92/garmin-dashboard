export interface Activity {
  id: number
  garmin_id: string
  name: string | null
  activity_type: string | null
  sport: string | null
  start_time: string
  duration_secs: number | null
  distance_meters: number | null
  calories: number | null
  avg_hr: number | null
  max_hr: number | null
  avg_speed: number | null
  elevation_gain: number | null
  avg_cadence: number | null
  avg_power: number | null
  vo2max: number | null
}

export interface ActivityRecord {
  id: number
  activity_id: number
  timestamp: string | null
  lat: number | null
  lon: number | null
  altitude: number | null
  heart_rate: number | null
  cadence: number | null
  speed: number | null
  power: number | null
  distance: number | null
}

export interface ActivityWithRecords {
  activity: Activity
  records: ActivityRecord[]
}

export interface DailyWellness {
  date: string
  steps: number | null
  floors: number | null
  active_calories: number | null
  resting_hr: number | null
  avg_stress: number | null
  body_battery_low: number | null
  body_battery_high: number | null
  sleep_seconds: number | null
  sleep_score: number | null
  hrv_night_avg: number | null
  spo2_avg: number | null
  intensity_minutes: number | null
}

export interface SleepSession {
  id: number
  date: string
  sleep_start: string | null
  sleep_end: string | null
  total_sleep_secs: number | null
  deep_sleep_secs: number | null
  light_sleep_secs: number | null
  rem_sleep_secs: number | null
  awake_secs: number | null
  sleep_score: number | null
}

export interface OverviewStats {
  total_activities: number
  total_distance_km: number
  total_calories: number
  avg_resting_hr: number | null
  recent_activities: Activity[]
  month_steps: number
  month_avg_sleep_hours: number
}
