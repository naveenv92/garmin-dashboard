use crate::models::{NewActivity, NewActivityRecord};
use anyhow::Result;
use std::path::Path;

pub struct GpxResult {
    pub activity: Option<NewActivity>,
    pub records: Vec<NewActivityRecord>,
}

pub fn parse(path: &Path) -> Result<GpxResult> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let gpx_data: gpx::Gpx = gpx::read(reader)?;

    let filename = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let garmin_id = format!("gpx_{}", filename);

    let mut records: Vec<NewActivityRecord> = Vec::new();
    let mut start_time: Option<String> = None;
    let mut total_distance: f64 = 0.0;
    let mut elevation_gain: f64 = 0.0;
    let mut elevation_loss: f64 = 0.0;
    let mut max_speed: f64 = 0.0;
    let mut prev_altitude: Option<f64> = None;
    let mut prev_point: Option<(f64, f64)> = None;

    for track in &gpx_data.tracks {
        for segment in &track.segments {
            for waypoint in &segment.points {
                let pt = waypoint.point();
                let lat = pt.y();
                let lon = pt.x();

                let timestamp = waypoint
                    .time
                    .and_then(|t| t.format().ok());

                if start_time.is_none() {
                    start_time = timestamp.clone();
                }

                let altitude = waypoint.elevation;

                if let (Some(prev_alt), Some(alt)) = (prev_altitude, altitude) {
                    if alt > prev_alt {
                        elevation_gain += alt - prev_alt;
                    } else {
                        elevation_loss += prev_alt - alt;
                    }
                }
                prev_altitude = altitude;

                if let Some((prev_lat, prev_lon)) = prev_point {
                    total_distance += haversine(prev_lat, prev_lon, lat, lon);
                }
                prev_point = Some((lat, lon));

                if let Some(speed) = waypoint.speed {
                    if speed > max_speed {
                        max_speed = speed;
                    }
                }

                records.push(NewActivityRecord {
                    activity_id: 0,
                    timestamp,
                    lat: Some(lat),
                    lon: Some(lon),
                    altitude,
                    heart_rate: None,
                    cadence: None,
                    speed: waypoint.speed,
                    power: None,
                    distance: None,
                });
            }
        }
    }

    let name = gpx_data
        .tracks
        .first()
        .and_then(|t| t.name.clone())
        .or_else(|| gpx_data.metadata.as_ref().and_then(|m| m.name.clone()));

    let activity_type = gpx_data
        .tracks
        .first()
        .and_then(|t| t.type_.clone());

    let activity = Some(NewActivity {
        garmin_id,
        name,
        activity_type,
        sport: None,
        start_time: start_time.unwrap_or_default(),
        duration_secs: None,
        distance_meters: if total_distance > 0.0 {
            Some(total_distance)
        } else {
            None
        },
        calories: None,
        avg_hr: None,
        max_hr: None,
        avg_speed: None,
        max_speed: if max_speed > 0.0 { Some(max_speed) } else { None },
        elevation_gain: if elevation_gain > 0.0 {
            Some(elevation_gain)
        } else {
            None
        },
        elevation_loss: if elevation_loss > 0.0 {
            Some(elevation_loss)
        } else {
            None
        },
        avg_cadence: None,
        avg_power: None,
        vo2max: None,
        source_file_hash: None,
    });

    Ok(GpxResult { activity, records })
}

fn haversine(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const R: f64 = 6_371_000.0; // Earth radius in meters
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    R * c
}
