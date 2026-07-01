use crate::models::{NewActivity, NewActivityRecord};
use anyhow::Result;
use std::path::Path;

pub struct FitResult {
    pub activity: Option<NewActivity>,
    pub records: Vec<NewActivityRecord>,
}

pub fn parse(path: &Path) -> Result<FitResult> {
    let mut fp = std::fs::File::open(path)?;
    let messages = fitparser::from_reader(&mut fp)?;

    let filename = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let mut garmin_id: Option<String> = None;
    let mut activity_type: Option<String> = None;
    let mut sport: Option<String> = None;
    let mut start_time: Option<String> = None;
    let mut duration_secs: Option<f64> = None;
    let mut distance_meters: Option<f64> = None;
    let mut calories: Option<i64> = None;
    let mut avg_hr: Option<i64> = None;
    let mut max_hr: Option<i64> = None;
    let mut avg_speed: Option<f64> = None;
    let mut elevation_gain: Option<f64> = None;
    let mut avg_cadence: Option<i64> = None;
    let mut avg_power: Option<i64> = None;
    let mut vo2max: Option<f64> = None;

    let mut records: Vec<NewActivityRecord> = Vec::new();

    for msg in &messages {
        match msg.kind() {
            fitparser::profile::MesgNum::FileId => {
                for field in msg.fields() {
                    if field.name() == "serial_number" || field.name() == "time_created" {
                        // Use filename as garmin_id fallback
                    }
                }
            }
            fitparser::profile::MesgNum::Activity => {
                for field in msg.fields() {
                    if field.name() == "timestamp" {
                        if let Some(ts) = value_to_string(&field.value()) {
                            if garmin_id.is_none() {
                                garmin_id = Some(format!("fit_{}", filename));
                            }
                            if start_time.is_none() {
                                start_time = Some(ts);
                            }
                        }
                    }
                }
            }
            fitparser::profile::MesgNum::Session => {
                for field in msg.fields() {
                    match field.name() {
                        "start_time" => {
                            start_time = value_to_string(&field.value());
                        }
                        "sport" => {
                            sport = value_to_string(&field.value());
                            if activity_type.is_none() {
                                activity_type = sport.clone();
                            }
                        }
                        "sub_sport" => {}
                        "total_elapsed_time" => {
                            duration_secs = value_to_f64(&field.value());
                        }
                        "total_distance" => {
                            distance_meters = value_to_f64(&field.value());
                        }
                        "total_calories" => {
                            calories = value_to_i64(&field.value());
                        }
                        "avg_heart_rate" => {
                            avg_hr = value_to_i64(&field.value());
                        }
                        "max_heart_rate" => {
                            max_hr = value_to_i64(&field.value());
                        }
                        "avg_speed" | "enhanced_avg_speed" => {
                            if avg_speed.is_none() {
                                avg_speed = value_to_f64(&field.value());
                            }
                        }
                        "total_ascent" => {
                            elevation_gain = value_to_f64(&field.value());
                        }
                        "avg_cadence" => {
                            avg_cadence = value_to_i64(&field.value());
                        }
                        "avg_power" => {
                            avg_power = value_to_i64(&field.value());
                        }
                        _ => {}
                    }
                }
                if garmin_id.is_none() {
                    garmin_id = Some(format!("fit_{}", filename));
                }
            }
            fitparser::profile::MesgNum::Record => {
                let mut rec = NewActivityRecord {
                    activity_id: 0,
                    timestamp: None,
                    lat: None,
                    lon: None,
                    altitude: None,
                    heart_rate: None,
                    cadence: None,
                    speed: None,
                    power: None,
                    distance: None,
                };

                for field in msg.fields() {
                    match field.name() {
                        "timestamp" => {
                            rec.timestamp = value_to_string(&field.value());
                        }
                        "position_lat" => {
                            rec.lat = value_to_f64(&field.value()).map(semicircles_to_deg);
                        }
                        "position_long" => {
                            rec.lon = value_to_f64(&field.value()).map(semicircles_to_deg);
                        }
                        "altitude" | "enhanced_altitude" => {
                            if rec.altitude.is_none() {
                                rec.altitude = value_to_f64(&field.value());
                            }
                        }
                        "heart_rate" => {
                            rec.heart_rate = value_to_i64(&field.value());
                        }
                        "cadence" => {
                            rec.cadence = value_to_i64(&field.value());
                        }
                        "speed" | "enhanced_speed" => {
                            if rec.speed.is_none() {
                                rec.speed = value_to_f64(&field.value());
                            }
                        }
                        "power" => {
                            rec.power = value_to_i64(&field.value());
                        }
                        "distance" => {
                            rec.distance = value_to_f64(&field.value());
                        }
                        _ => {}
                    }
                }

                records.push(rec);
            }
            _ => {}
        }
    }

    let activity = garmin_id.map(|id| NewActivity {
        garmin_id: id,
        name: None,
        activity_type,
        sport,
        start_time: start_time.unwrap_or_default(),
        duration_secs,
        distance_meters,
        calories,
        avg_hr,
        max_hr,
        avg_speed,
        elevation_gain,
        avg_cadence,
        avg_power,
        vo2max,
        source_file_hash: None,
    });

    Ok(FitResult { activity, records })
}

fn semicircles_to_deg(v: f64) -> f64 {
    v * (180.0 / 2_147_483_648.0)
}

fn value_to_f64(v: &fitparser::Value) -> Option<f64> {
    match v {
        fitparser::Value::Float64(f) => Some(*f),
        fitparser::Value::Float32(f) => Some(*f as f64),
        fitparser::Value::SInt32(i) => Some(*i as f64),
        fitparser::Value::UInt32(i) => Some(*i as f64),
        fitparser::Value::SInt16(i) => Some(*i as f64),
        fitparser::Value::UInt16(i) => Some(*i as f64),
        fitparser::Value::UInt8(i) => Some(*i as f64),
        fitparser::Value::SInt8(i) => Some(*i as f64),
        fitparser::Value::SInt64(i) => Some(*i as f64),
        fitparser::Value::UInt64(i) => Some(*i as f64),
        _ => None,
    }
}

fn value_to_i64(v: &fitparser::Value) -> Option<i64> {
    value_to_f64(v).map(|f| f as i64)
}

fn value_to_string(v: &fitparser::Value) -> Option<String> {
    match v {
        fitparser::Value::String(s) => Some(s.clone()),
        fitparser::Value::Timestamp(dt) => Some(dt.to_rfc3339()),
        other => {
            let s = format!("{:?}", other);
            if s == "Invalid" || s.is_empty() {
                None
            } else {
                Some(s)
            }
        }
    }
}
