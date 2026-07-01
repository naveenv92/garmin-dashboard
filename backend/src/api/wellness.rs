use crate::models::{DailyWellness, SleepSession};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sqlx::{Row, SqlitePool};

#[derive(Deserialize)]
pub struct DateRangeQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub limit: Option<i64>,
}

pub async fn daily(
    State(pool): State<SqlitePool>,
    Query(q): Query<DateRangeQuery>,
) -> Result<Json<Vec<DailyWellness>>, StatusCode> {
    let limit = q.limit.unwrap_or(90).min(365);

    let rows = sqlx::query(
        r#"SELECT date, steps, floors, active_calories, resting_hr, avg_stress,
                  body_battery_low, body_battery_high, sleep_seconds, sleep_score,
                  hrv_night_avg, spo2_avg, intensity_minutes
           FROM daily_wellness
           WHERE (? IS NULL OR date >= ?)
             AND (? IS NULL OR date <= ?)
           ORDER BY date DESC
           LIMIT ?"#,
    )
    .bind(&q.from)
    .bind(&q.from)
    .bind(&q.to)
    .bind(&q.to)
    .bind(limit)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let wellness = rows
        .into_iter()
        .map(|r| DailyWellness {
            date: r.get("date"),
            steps: r.get("steps"),
            floors: r.get("floors"),
            active_calories: r.get("active_calories"),
            resting_hr: r.get("resting_hr"),
            avg_stress: r.get("avg_stress"),
            body_battery_low: r.get("body_battery_low"),
            body_battery_high: r.get("body_battery_high"),
            sleep_seconds: r.get("sleep_seconds"),
            sleep_score: r.get("sleep_score"),
            hrv_night_avg: r.get("hrv_night_avg"),
            spo2_avg: r.get("spo2_avg"),
            intensity_minutes: r.get("intensity_minutes"),
        })
        .collect();

    Ok(Json(wellness))
}

pub async fn sleep_list(
    State(pool): State<SqlitePool>,
    Query(q): Query<DateRangeQuery>,
) -> Result<Json<Vec<SleepSession>>, StatusCode> {
    let limit = q.limit.unwrap_or(90).min(365);

    let rows = sqlx::query(
        r#"SELECT id, date, sleep_start, sleep_end, total_sleep_secs,
                  deep_sleep_secs, light_sleep_secs, rem_sleep_secs,
                  awake_secs, sleep_score
           FROM sleep_sessions
           WHERE (? IS NULL OR date >= ?)
             AND (? IS NULL OR date <= ?)
           ORDER BY date DESC
           LIMIT ?"#,
    )
    .bind(&q.from)
    .bind(&q.from)
    .bind(&q.to)
    .bind(&q.to)
    .bind(limit)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let sessions = rows
        .into_iter()
        .map(|r| SleepSession {
            id: r.get("id"),
            date: r.get("date"),
            sleep_start: r.get("sleep_start"),
            sleep_end: r.get("sleep_end"),
            total_sleep_secs: r.get("total_sleep_secs"),
            deep_sleep_secs: r.get("deep_sleep_secs"),
            light_sleep_secs: r.get("light_sleep_secs"),
            rem_sleep_secs: r.get("rem_sleep_secs"),
            awake_secs: r.get("awake_secs"),
            sleep_score: r.get("sleep_score"),
        })
        .collect();

    Ok(Json(sessions))
}
