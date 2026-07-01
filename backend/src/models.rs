use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct NewActivity {
    pub garmin_id: String,
    pub name: Option<String>,
    pub activity_type: Option<String>,
    pub sport: Option<String>,
    pub start_time: String,
    pub duration_secs: Option<f64>,
    pub distance_meters: Option<f64>,
    pub calories: Option<i64>,
    pub avg_hr: Option<i64>,
    pub max_hr: Option<i64>,
    pub avg_speed: Option<f64>,
    pub elevation_gain: Option<f64>,
    pub avg_cadence: Option<i64>,
    pub avg_power: Option<i64>,
    pub vo2max: Option<f64>,
    pub source_file_hash: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NewActivityRecord {
    pub activity_id: i64,
    pub timestamp: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub altitude: Option<f64>,
    pub heart_rate: Option<i64>,
    pub cadence: Option<i64>,
    pub speed: Option<f64>,
    pub power: Option<i64>,
    pub distance: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct NewDailyWellness {
    pub date: NaiveDate,
    pub steps: Option<i64>,
    pub floors: Option<i64>,
    pub active_calories: Option<i64>,
    pub resting_hr: Option<i64>,
    pub avg_stress: Option<i64>,
    pub body_battery_low: Option<i64>,
    pub body_battery_high: Option<i64>,
    pub sleep_seconds: Option<i64>,
    pub sleep_score: Option<i64>,
    pub hrv_night_avg: Option<f64>,
    pub spo2_avg: Option<f64>,
    pub intensity_minutes: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct NewSleepSession {
    pub date: NaiveDate,
    pub sleep_start: Option<NaiveDateTime>,
    pub sleep_end: Option<NaiveDateTime>,
    pub total_sleep_secs: Option<i64>,
    pub deep_sleep_secs: Option<i64>,
    pub light_sleep_secs: Option<i64>,
    pub rem_sleep_secs: Option<i64>,
    pub awake_secs: Option<i64>,
    pub sleep_score: Option<i64>,
}

// API response types
#[derive(Debug, Serialize, Deserialize)]
pub struct Activity {
    pub id: i64,
    pub garmin_id: String,
    pub name: Option<String>,
    pub activity_type: Option<String>,
    pub sport: Option<String>,
    pub start_time: String,
    pub duration_secs: Option<f64>,
    pub distance_meters: Option<f64>,
    pub calories: Option<i64>,
    pub avg_hr: Option<i64>,
    pub max_hr: Option<i64>,
    pub avg_speed: Option<f64>,
    pub elevation_gain: Option<f64>,
    pub avg_cadence: Option<i64>,
    pub avg_power: Option<i64>,
    pub vo2max: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityRecord {
    pub id: i64,
    pub activity_id: i64,
    pub timestamp: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub altitude: Option<f64>,
    pub heart_rate: Option<i64>,
    pub cadence: Option<i64>,
    pub speed: Option<f64>,
    pub power: Option<i64>,
    pub distance: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityWithRecords {
    pub activity: Activity,
    pub records: Vec<ActivityRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DailyWellness {
    pub date: String,
    pub steps: Option<i64>,
    pub floors: Option<i64>,
    pub active_calories: Option<i64>,
    pub resting_hr: Option<i64>,
    pub avg_stress: Option<i64>,
    pub body_battery_low: Option<i64>,
    pub body_battery_high: Option<i64>,
    pub sleep_seconds: Option<i64>,
    pub sleep_score: Option<i64>,
    pub hrv_night_avg: Option<f64>,
    pub spo2_avg: Option<f64>,
    pub intensity_minutes: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SleepSession {
    pub id: i64,
    pub date: String,
    pub sleep_start: Option<String>,
    pub sleep_end: Option<String>,
    pub total_sleep_secs: Option<i64>,
    pub deep_sleep_secs: Option<i64>,
    pub light_sleep_secs: Option<i64>,
    pub rem_sleep_secs: Option<i64>,
    pub awake_secs: Option<i64>,
    pub sleep_score: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OverviewStats {
    pub total_activities: i64,
    pub total_distance_km: f64,
    pub total_calories: i64,
    pub avg_resting_hr: Option<f64>,
    pub recent_activities: Vec<Activity>,
    pub month_steps: i64,
    pub month_avg_sleep_hours: f64,
}
