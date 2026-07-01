use crate::models::{Activity, ActivityRecord, ActivityWithRecords, OverviewStats};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Datelike;
use serde::Deserialize;
use sqlx::{Row, SqlitePool};

#[derive(Deserialize)]
pub struct ActivityQuery {
    pub activity_type: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list(
    State(pool): State<SqlitePool>,
    Query(q): Query<ActivityQuery>,
) -> Result<Json<Vec<Activity>>, StatusCode> {
    let limit = q.limit.unwrap_or(50).min(500);
    let offset = q.offset.unwrap_or(0);

    let rows = sqlx::query(
        r#"SELECT id, garmin_id, name, activity_type, sport, start_time,
                  duration_secs, distance_meters, calories, avg_hr, max_hr,
                  avg_speed, elevation_gain, avg_cadence, avg_power, vo2max
           FROM activities
           WHERE (? IS NULL OR LOWER(activity_type) LIKE '%' || LOWER(?) || '%')
             AND (? IS NULL OR start_time >= ?)
             AND (? IS NULL OR start_time <= ?)
           ORDER BY start_time DESC
           LIMIT ? OFFSET ?"#,
    )
    .bind(&q.activity_type)
    .bind(&q.activity_type)
    .bind(&q.from)
    .bind(&q.from)
    .bind(&q.to)
    .bind(&q.to)
    .bind(limit)
    .bind(offset)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let activities = rows
        .into_iter()
        .map(row_to_activity)
        .collect();

    Ok(Json(activities))
}

pub async fn get_one(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ActivityWithRecords>, StatusCode> {
    let row = sqlx::query(
        r#"SELECT id, garmin_id, name, activity_type, sport, start_time,
                  duration_secs, distance_meters, calories, avg_hr, max_hr,
                  avg_speed, elevation_gain, avg_cadence, avg_power, vo2max
           FROM activities WHERE id = ?"#,
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let activity = row_to_activity(row);

    let record_rows = sqlx::query(
        r#"SELECT id, activity_id, timestamp, lat, lon, altitude,
                  heart_rate, cadence, speed, power, distance
           FROM activity_records WHERE activity_id = ?
           ORDER BY timestamp ASC"#,
    )
    .bind(id)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let records = record_rows
        .into_iter()
        .map(|r| ActivityRecord {
            id: r.get("id"),
            activity_id: r.get("activity_id"),
            timestamp: r.get("timestamp"),
            lat: r.get("lat"),
            lon: r.get("lon"),
            altitude: r.get("altitude"),
            heart_rate: r.get("heart_rate"),
            cadence: r.get("cadence"),
            speed: r.get("speed"),
            power: r.get("power"),
            distance: r.get("distance"),
        })
        .collect();

    Ok(Json(ActivityWithRecords { activity, records }))
}

pub async fn overview(
    State(pool): State<SqlitePool>,
) -> Result<Json<OverviewStats>, StatusCode> {
    // Garmin dates are calendar days in the user's local timezone. SQLite's
    // date('now') is UTC, which drifts a day off local "today" for large
    // parts of the day in any timezone behind UTC — compute boundaries in
    // local time instead.
    let today = chrono::Local::now().date_naive();
    let thirty_days_ago = (today - chrono::Duration::days(30)).to_string();
    let start_of_month = today.with_day(1).unwrap_or(today).to_string();

    let totals = sqlx::query(
        r#"SELECT
             COUNT(*) as total_activities,
             COALESCE(SUM(distance_meters), 0.0) as total_distance,
             COALESCE(SUM(calories), 0) as total_calories
           FROM activities"#,
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let avg_resting = sqlx::query(
        r#"SELECT AVG(CAST(resting_hr AS REAL)) as avg_resting_hr
           FROM daily_wellness
           WHERE resting_hr IS NOT NULL
             AND date >= ?"#,
    )
    .bind(&thirty_days_ago)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let recent_rows = sqlx::query(
        r#"SELECT id, garmin_id, name, activity_type, sport, start_time,
                  duration_secs, distance_meters, calories, avg_hr, max_hr,
                  avg_speed, elevation_gain, avg_cadence, avg_power, vo2max
           FROM activities ORDER BY start_time DESC LIMIT 5"#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let month_steps = sqlx::query(
        r#"SELECT COALESCE(SUM(steps), 0) as total_steps
           FROM daily_wellness
           WHERE date >= ?"#,
    )
    .bind(&start_of_month)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let avg_sleep = sqlx::query(
        r#"SELECT COALESCE(AVG(CAST(total_sleep_secs AS REAL)), 0.0) as avg_sleep
           FROM sleep_sessions
           WHERE date >= ?
             AND total_sleep_secs IS NOT NULL"#,
    )
    .bind(&thirty_days_ago)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_activities: i64 = totals.get("total_activities");
    let total_distance: f64 = totals.get("total_distance");
    let total_calories: i64 = totals.get("total_calories");

    Ok(Json(OverviewStats {
        total_activities,
        total_distance_km: total_distance / 1000.0,
        total_calories,
        avg_resting_hr: avg_resting.get("avg_resting_hr"),
        recent_activities: recent_rows.into_iter().map(row_to_activity).collect(),
        month_steps: month_steps.get("total_steps"),
        month_avg_sleep_hours: {
            let avg: f64 = avg_sleep.get("avg_sleep");
            avg / 3600.0
        },
    }))
}

fn row_to_activity(r: sqlx::sqlite::SqliteRow) -> Activity {
    Activity {
        id: r.get("id"),
        garmin_id: r.get("garmin_id"),
        name: r.get("name"),
        activity_type: r.get("activity_type"),
        sport: r.get("sport"),
        start_time: r.get("start_time"),
        duration_secs: r.get("duration_secs"),
        distance_meters: r.get("distance_meters"),
        calories: r.get("calories"),
        avg_hr: r.get("avg_hr"),
        max_hr: r.get("max_hr"),
        avg_speed: r.get("avg_speed"),
        elevation_gain: r.get("elevation_gain"),
        avg_cadence: r.get("avg_cadence"),
        avg_power: r.get("avg_power"),
        vo2max: r.get("vo2max"),
    }
}
