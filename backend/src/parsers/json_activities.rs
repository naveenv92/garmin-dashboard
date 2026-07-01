use crate::models::NewActivity;
use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawActivity {
    activity_id: Option<serde_json::Value>,
    activity_name: Option<String>,
    #[serde(rename = "activityType")]
    activity_type: Option<ActivityType>,
    start_time_local: Option<String>,
    start_time_gmt: Option<String>,
    duration: Option<f64>,
    distance: Option<f64>,
    calories: Option<f64>,
    average_hr: Option<f64>,
    max_hr: Option<f64>,
    average_speed: Option<f64>,
    elevation_gain: Option<f64>,
    average_cadence: Option<f64>,
    average_power: Option<f64>,
    vo2_max_value: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ActivityType {
    type_key: Option<String>,
    parent_type_id: Option<i64>,
}

pub fn parse(json: &str) -> Result<Vec<NewActivity>> {
    let raw: Vec<RawActivity> = serde_json::from_str(json)
        .or_else(|_| {
            // Some exports wrap in an object
            let v: serde_json::Value = serde_json::from_str(json)?;
            if let Some(arr) = v.get("summarizedActivitiesExport") {
                serde_json::from_value(arr.clone())
            } else {
                Err(serde::de::Error::custom("no activities array found"))
            }
        })?;

    let activities = raw
        .into_iter()
        .filter_map(|r| {
            let garmin_id = r.activity_id.as_ref().map(|v| match v {
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            })?;

            let start_time = r
                .start_time_local
                .or(r.start_time_gmt)
                .unwrap_or_default();

            if start_time.is_empty() {
                return None;
            }

            let activity_type = r
                .activity_type
                .as_ref()
                .and_then(|t| t.type_key.clone())
                .map(|k| capitalize(&k));

            Some(NewActivity {
                garmin_id,
                name: r.activity_name,
                activity_type,
                sport: None,
                start_time,
                duration_secs: r.duration,
                distance_meters: r.distance,
                calories: r.calories.map(|c| c as i64),
                avg_hr: r.average_hr.map(|h| h as i64),
                max_hr: r.max_hr.map(|h| h as i64),
                avg_speed: r.average_speed,
                elevation_gain: r.elevation_gain,
                avg_cadence: r.average_cadence.map(|c| c as i64),
                avg_power: r.average_power.map(|p| p as i64),
                vo2max: r.vo2_max_value,
                source_file_hash: None,
            })
        })
        .collect();

    Ok(activities)
}

fn capitalize(s: &str) -> String {
    let mut chars = s.replace('_', " ").chars().collect::<Vec<_>>();
    if let Some(c) = chars.first_mut() {
        *c = c.to_uppercase().next().unwrap_or(*c);
    }
    chars.into_iter().collect()
}
