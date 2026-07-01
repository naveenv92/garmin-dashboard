use crate::db;
use crate::models::{
    NewActivity, NewActivityLap, NewActivityRecord, NewDailyWellness, NewSleepSession,
};
use crate::parsers;
use anyhow::Result;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use std::path::Path;
use walkdir::WalkDir;

pub async fn import(pool: &SqlitePool, root: &Path) -> Result<()> {
    if !root.exists() {
        anyhow::bail!("Path does not exist: {}", root.display());
    }

    println!("Scanning: {}", root.display());

    let mut total_activities = 0usize;
    let mut total_wellness_days = 0usize;
    let mut total_sleep_sessions = 0usize;
    let mut total_records = 0usize;
    let mut skipped = 0usize;
    let mut errors = 0usize;

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let path_str = path.to_string_lossy().to_string();

        let hash = match hash_file(path) {
            Ok(h) => h,
            Err(e) => {
                eprintln!("  [warn] Could not hash {}: {}", path_str, e);
                errors += 1;
                continue;
            }
        };

        if db::file_already_imported(pool, &hash).await? {
            skipped += 1;
            continue;
        }

        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();

        let result = match filename.as_str() {
            name if name == "summarizedactivities.json"
                || name == "summarized_activities.json" =>
            {
                let data = std::fs::read_to_string(path)?;
                parsers::json_activities::parse(&data)
                    .map(|acts| ParseResult::Activities(acts))
            }
            name if name.ends_with(".fit") => {
                parsers::fit::parse(path).map(|r| ParseResult::FitActivity(r))
            }
            name if name.ends_with(".gpx") => {
                parsers::gpx::parse(path).map(|r| ParseResult::GpxActivity(r))
            }
            name if name.ends_with(".json") && is_uds_daily(name) => {
                let data = std::fs::read_to_string(path)?;
                parsers::json_wellness::parse_uds_daily(&data)
                    .map(|rows| ParseResult::Wellness(rows))
            }
            name if name.ends_with(".json") && is_sleep(name) => {
                let data = std::fs::read_to_string(path)?;
                parsers::json_wellness::parse_sleep_json(&data)
                    .map(|rows| ParseResult::Sleep(rows))
            }
            name if name.ends_with(".csv") && is_daily_summary(name) => {
                let data = std::fs::read_to_string(path)?;
                parsers::csv_wellness::parse_daily_summary(&data)
                    .map(|rows| ParseResult::Wellness(rows))
            }
            name if name.ends_with(".csv") && is_sleep(name) => {
                let data = std::fs::read_to_string(path)?;
                parsers::csv_wellness::parse_sleep(&data)
                    .map(|rows| ParseResult::Sleep(rows))
            }
            name if name.ends_with(".csv") && is_body_battery(name) => {
                let data = std::fs::read_to_string(path)?;
                parsers::csv_wellness::parse_body_battery(pool, &data).await?;
                Ok(ParseResult::Skipped)
            }
            name if name.ends_with(".csv") && is_hrv(name) => {
                let data = std::fs::read_to_string(path)?;
                parsers::csv_wellness::parse_hrv(pool, &data).await?;
                Ok(ParseResult::Skipped)
            }
            name if name.ends_with(".csv") && is_stress(name) => {
                let data = std::fs::read_to_string(path)?;
                parsers::csv_wellness::parse_stress(pool, &data).await?;
                Ok(ParseResult::Skipped)
            }
            _ => Ok(ParseResult::Skipped),
        };

        match result {
            Ok(ParseResult::Activities(acts)) => {
                let n = acts.len();
                insert_activities(pool, acts, &hash).await?;
                total_activities += n;
                db::record_imported_file(pool, &path_str, &hash).await?;
                println!("  [json] {} activities from {}", n, filename);
            }
            Ok(ParseResult::FitActivity(fit_result)) => {
                if let Some(act) = fit_result.activity {
                    let result = insert_activity(pool, act, &hash).await?;
                    if let Some(activity_id) = result {
                        let n = fit_result.records.len();
                        insert_records(pool, activity_id, fit_result.records).await?;
                        insert_laps(pool, activity_id, fit_result.laps).await?;
                        total_activities += 1;
                        total_records += n;
                    }
                }
                db::record_imported_file(pool, &path_str, &hash).await?;
            }
            Ok(ParseResult::GpxActivity(gpx_result)) => {
                if let Some(act) = gpx_result.activity {
                    let result = insert_activity(pool, act, &hash).await?;
                    if let Some(activity_id) = result {
                        let n = gpx_result.records.len();
                        insert_records(pool, activity_id, gpx_result.records).await?;
                        total_activities += 1;
                        total_records += n;
                    }
                }
                db::record_imported_file(pool, &path_str, &hash).await?;
            }
            Ok(ParseResult::Wellness(rows)) => {
                let n = rows.len();
                insert_wellness(pool, rows).await?;
                total_wellness_days += n;
                db::record_imported_file(pool, &path_str, &hash).await?;
                println!("  [csv]  {} wellness days from {}", n, filename);
            }
            Ok(ParseResult::Sleep(rows)) => {
                let n = rows.len();
                insert_sleep(pool, rows).await?;
                total_sleep_sessions += n;
                db::record_imported_file(pool, &path_str, &hash).await?;
                println!("  [csv]  {} sleep sessions from {}", n, filename);
            }
            Ok(ParseResult::Skipped) => {
                db::record_imported_file(pool, &path_str, &hash).await?;
            }
            Err(e) => {
                eprintln!("  [warn] Failed to parse {}: {}", filename, e);
                errors += 1;
            }
        }
    }

    println!();
    println!("Import complete:");
    println!("  Activities:     {}", total_activities);
    println!("  Track records:  {}", total_records);
    println!("  Wellness days:  {}", total_wellness_days);
    println!("  Sleep sessions: {}", total_sleep_sessions);
    println!("  Skipped (seen): {}", skipped);
    if errors > 0 {
        println!("  Errors:         {}", errors);
    }

    Ok(())
}

enum ParseResult {
    Activities(Vec<NewActivity>),
    FitActivity(parsers::fit::FitResult),
    GpxActivity(parsers::gpx::GpxResult),
    Wellness(Vec<NewDailyWellness>),
    Sleep(Vec<NewSleepSession>),
    Skipped,
}

fn hash_file(path: &Path) -> Result<String> {
    let data = std::fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    Ok(hex::encode(hasher.finalize()))
}

fn is_daily_summary(name: &str) -> bool {
    name.contains("dailysummary") || name.contains("daily_summary")
}

fn is_uds_daily(name: &str) -> bool {
    name.contains("udsfile")
}

fn is_sleep(name: &str) -> bool {
    name.contains("sleepdata") || name.contains("sleep_data") || name.contains("sleepscore")
}

fn is_body_battery(name: &str) -> bool {
    name.contains("bodybattery") || name.contains("body_battery")
}

fn is_hrv(name: &str) -> bool {
    name.contains("hrv") || name.contains("heartratevariability")
}

fn is_stress(name: &str) -> bool {
    name.contains("stress")
}

async fn insert_activity(
    pool: &SqlitePool,
    act: NewActivity,
    hash: &str,
) -> Result<Option<i64>> {
    let result = sqlx::query(
        r#"INSERT OR IGNORE INTO activities
           (garmin_id, name, activity_type, sport, start_time,
            duration_secs, distance_meters, calories, avg_hr, max_hr,
            avg_speed, max_speed, elevation_gain, elevation_loss,
            avg_cadence, avg_power, vo2max, source_file_hash)
           VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)"#,
    )
    .bind(&act.garmin_id)
    .bind(&act.name)
    .bind(&act.activity_type)
    .bind(&act.sport)
    .bind(&act.start_time)
    .bind(act.duration_secs)
    .bind(act.distance_meters)
    .bind(act.calories)
    .bind(act.avg_hr)
    .bind(act.max_hr)
    .bind(act.avg_speed)
    .bind(act.max_speed)
    .bind(act.elevation_gain)
    .bind(act.elevation_loss)
    .bind(act.avg_cadence)
    .bind(act.avg_power)
    .bind(act.vo2max)
    .bind(hash)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Ok(None);
    }

    let row: (i64,) =
        sqlx::query_as("SELECT id FROM activities WHERE garmin_id = ?")
            .bind(&act.garmin_id)
            .fetch_one(pool)
            .await?;
    Ok(Some(row.0))
}

async fn insert_activities(
    pool: &SqlitePool,
    acts: Vec<NewActivity>,
    hash: &str,
) -> Result<()> {
    for act in acts {
        let _ = insert_activity(pool, act, hash).await?;
    }
    Ok(())
}

async fn insert_records(
    pool: &SqlitePool,
    activity_id: i64,
    records: Vec<NewActivityRecord>,
) -> Result<()> {
    for rec in records {
        sqlx::query(
            r#"INSERT INTO activity_records
               (activity_id, timestamp, lat, lon, altitude, heart_rate, cadence, speed, power, distance)
               VALUES (?,?,?,?,?,?,?,?,?,?)"#,
        )
        .bind(activity_id)
        .bind(&rec.timestamp)
        .bind(rec.lat)
        .bind(rec.lon)
        .bind(rec.altitude)
        .bind(rec.heart_rate)
        .bind(rec.cadence)
        .bind(rec.speed)
        .bind(rec.power)
        .bind(rec.distance)
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn insert_laps(
    pool: &SqlitePool,
    activity_id: i64,
    laps: Vec<NewActivityLap>,
) -> Result<()> {
    for lap in laps {
        sqlx::query(
            r#"INSERT INTO activity_laps
               (activity_id, lap_index, start_time, end_time, duration_secs,
                distance_meters, elevation_gain, elevation_loss, max_speed,
                avg_speed, avg_hr, max_hr, calories, min_altitude, max_altitude)
               VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)"#,
        )
        .bind(activity_id)
        .bind(lap.lap_index)
        .bind(&lap.start_time)
        .bind(&lap.end_time)
        .bind(lap.duration_secs)
        .bind(lap.distance_meters)
        .bind(lap.elevation_gain)
        .bind(lap.elevation_loss)
        .bind(lap.max_speed)
        .bind(lap.avg_speed)
        .bind(lap.avg_hr)
        .bind(lap.max_hr)
        .bind(lap.calories)
        .bind(lap.min_altitude)
        .bind(lap.max_altitude)
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn insert_wellness(pool: &SqlitePool, rows: Vec<NewDailyWellness>) -> Result<()> {
    for row in rows {
        sqlx::query(
            r#"INSERT OR IGNORE INTO daily_wellness
               (date, steps, floors, active_calories, resting_hr, avg_stress,
                body_battery_low, body_battery_high, sleep_seconds, sleep_score,
                hrv_night_avg, spo2_avg, intensity_minutes)
               VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?)"#,
        )
        .bind(row.date.to_string())
        .bind(row.steps)
        .bind(row.floors)
        .bind(row.active_calories)
        .bind(row.resting_hr)
        .bind(row.avg_stress)
        .bind(row.body_battery_low)
        .bind(row.body_battery_high)
        .bind(row.sleep_seconds)
        .bind(row.sleep_score)
        .bind(row.hrv_night_avg)
        .bind(row.spo2_avg)
        .bind(row.intensity_minutes)
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn insert_sleep(pool: &SqlitePool, rows: Vec<NewSleepSession>) -> Result<()> {
    for row in rows {
        sqlx::query(
            r#"INSERT OR IGNORE INTO sleep_sessions
               (date, sleep_start, sleep_end, total_sleep_secs, deep_sleep_secs,
                light_sleep_secs, rem_sleep_secs, awake_secs, sleep_score)
               VALUES (?,?,?,?,?,?,?,?,?)"#,
        )
        .bind(row.date.to_string())
        .bind(row.sleep_start.map(|dt| dt.to_string()))
        .bind(row.sleep_end.map(|dt| dt.to_string()))
        .bind(row.total_sleep_secs)
        .bind(row.deep_sleep_secs)
        .bind(row.light_sleep_secs)
        .bind(row.rem_sleep_secs)
        .bind(row.awake_secs)
        .bind(row.sleep_score)
        .execute(pool)
        .await?;
    }
    Ok(())
}
