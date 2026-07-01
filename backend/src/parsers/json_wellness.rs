use crate::models::{NewDailyWellness, NewSleepSession};
use anyhow::Result;
use chrono::{NaiveDate, NaiveDateTime};
use serde::Deserialize;

// Garmin Connect export format (DI_CONNECT/DI-Connect-Aggregator/UDSFile_*.json):
// an array of per-day "User Daily Summary" records.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UdsRecord {
    calendar_date: NaiveDate,
    total_steps: Option<i64>,
    active_kilocalories: Option<f64>,
    resting_heart_rate: Option<i64>,
    floors_ascended_in_meters: Option<f64>,
    moderate_intensity_minutes: Option<i64>,
    vigorous_intensity_minutes: Option<i64>,
    all_day_stress: Option<AllDayStress>,
    body_battery: Option<BodyBattery>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AllDayStress {
    aggregator_list: Option<Vec<StressAggregator>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StressAggregator {
    #[serde(rename = "type")]
    stress_type: String,
    average_stress_level: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BodyBattery {
    body_battery_stat_list: Option<Vec<BodyBatteryStat>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BodyBatteryStat {
    body_battery_stat_type: String,
    stats_value: Option<i64>,
}

// One Garmin "floor" is 10ft / 3.048m of ascent.
const METERS_PER_FLOOR: f64 = 3.048;

pub fn parse_uds_daily(data: &str) -> Result<Vec<NewDailyWellness>> {
    let records: Vec<UdsRecord> = serde_json::from_str(data)?;
    let mut rows = Vec::with_capacity(records.len());

    for rec in records {
        // Garmin uses negative sentinel values (-1, -2) to mean "no reading".
        let avg_stress = rec
            .all_day_stress
            .as_ref()
            .and_then(|s| {
                s.aggregator_list.as_ref().and_then(|list| {
                    list.iter()
                        .find(|a| a.stress_type == "TOTAL")
                        .and_then(|a| a.average_stress_level)
                })
            })
            .filter(|&v| v >= 0);

        let mut body_battery_low = None;
        let mut body_battery_high = None;
        if let Some(list) = rec.body_battery.as_ref().and_then(|b| b.body_battery_stat_list.as_ref()) {
            for stat in list {
                match stat.body_battery_stat_type.as_str() {
                    "LOWEST" => body_battery_low = stat.stats_value,
                    "HIGHEST" => body_battery_high = stat.stats_value,
                    _ => {}
                }
            }
        }

        let floors = rec
            .floors_ascended_in_meters
            .map(|m| (m / METERS_PER_FLOOR).round() as i64);

        let intensity_minutes = match (rec.moderate_intensity_minutes, rec.vigorous_intensity_minutes) {
            (None, None) => None,
            (m, v) => Some(m.unwrap_or(0) + 2 * v.unwrap_or(0)),
        };

        rows.push(NewDailyWellness {
            date: rec.calendar_date,
            steps: rec.total_steps,
            floors,
            active_calories: rec.active_kilocalories.map(|c| c.round() as i64),
            resting_hr: rec.resting_heart_rate,
            avg_stress,
            body_battery_low,
            body_battery_high,
            sleep_seconds: None,
            sleep_score: None,
            hrv_night_avg: None,
            spo2_avg: None,
            intensity_minutes,
        });
    }

    Ok(rows)
}

// Garmin Connect export format (DI_CONNECT/DI-Connect-Wellness/*_sleepData.json):
// an array of per-night sleep records.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SleepRecord {
    calendar_date: NaiveDate,
    #[serde(rename = "sleepStartTimestampGMT")]
    sleep_start_timestamp_gmt: Option<String>,
    #[serde(rename = "sleepEndTimestampGMT")]
    sleep_end_timestamp_gmt: Option<String>,
    deep_sleep_seconds: Option<i64>,
    light_sleep_seconds: Option<i64>,
    rem_sleep_seconds: Option<i64>,
    awake_sleep_seconds: Option<i64>,
    sleep_scores: Option<SleepScores>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SleepScores {
    overall_score: Option<i64>,
}

pub fn parse_sleep_json(data: &str) -> Result<Vec<NewSleepSession>> {
    let records: Vec<SleepRecord> = serde_json::from_str(data)?;
    let mut rows = Vec::with_capacity(records.len());

    for rec in records {
        let deep = rec.deep_sleep_seconds;
        let light = rec.light_sleep_seconds;
        let rem = rec.rem_sleep_seconds;
        let awake = rec.awake_sleep_seconds;

        let total = {
            let sum = deep.unwrap_or(0) + light.unwrap_or(0) + rem.unwrap_or(0);
            if sum > 0 { Some(sum) } else { None }
        };

        rows.push(NewSleepSession {
            date: rec.calendar_date,
            sleep_start: rec
                .sleep_start_timestamp_gmt
                .as_deref()
                .and_then(parse_gmt_datetime),
            sleep_end: rec
                .sleep_end_timestamp_gmt
                .as_deref()
                .and_then(parse_gmt_datetime),
            total_sleep_secs: total,
            deep_sleep_secs: deep,
            light_sleep_secs: light,
            rem_sleep_secs: rem,
            awake_secs: awake,
            sleep_score: rec.sleep_scores.and_then(|s| s.overall_score),
        });
    }

    Ok(rows)
}

fn parse_gmt_datetime(s: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f").ok()
}
