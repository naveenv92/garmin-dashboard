use crate::models::{NewDailyWellness, NewSleepSession};
use anyhow::Result;
use chrono::NaiveDate;
use sqlx::SqlitePool;

pub fn parse_daily_summary(data: &str) -> Result<Vec<NewDailyWellness>> {
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(data.as_bytes());

    let headers = reader.headers()?.clone();
    let headers_lower: Vec<String> = headers
        .iter()
        .map(|h| h.to_lowercase().trim().to_string())
        .collect();

    let col = |names: &[&str]| -> Option<usize> {
        names.iter().find_map(|n| {
            headers_lower
                .iter()
                .position(|h| h.contains(n))
        })
    };

    let idx_date = col(&["date"]);
    let idx_steps = col(&["steps"]);
    let idx_floors = col(&["floors"]);
    let idx_active_cal = col(&["active calor", "active_calor"]);
    let idx_resting_hr = col(&["resting heart", "resting_heart"]);
    let idx_intensity = col(&["intensity minutes", "intensity_minutes"]);

    let mut rows = Vec::new();

    for result in reader.records() {
        let record = result?;

        let date_str = idx_date
            .and_then(|i| record.get(i))
            .unwrap_or("")
            .trim()
            .to_string();

        let date = parse_date(&date_str);
        let Some(date) = date else { continue };

        rows.push(NewDailyWellness {
            date,
            steps: idx_steps.and_then(|i| parse_i64(record.get(i))),
            floors: idx_floors.and_then(|i| parse_i64(record.get(i))),
            active_calories: idx_active_cal.and_then(|i| parse_i64(record.get(i))),
            resting_hr: idx_resting_hr.and_then(|i| parse_i64(record.get(i))),
            avg_stress: None,
            body_battery_low: None,
            body_battery_high: None,
            sleep_seconds: None,
            sleep_score: None,
            hrv_night_avg: None,
            spo2_avg: None,
            intensity_minutes: idx_intensity.and_then(|i| parse_i64(record.get(i))),
        });
    }

    Ok(rows)
}

pub fn parse_sleep(data: &str) -> Result<Vec<NewSleepSession>> {
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(data.as_bytes());

    let headers = reader.headers()?.clone();
    let headers_lower: Vec<String> = headers
        .iter()
        .map(|h| h.to_lowercase().trim().to_string())
        .collect();

    let col = |names: &[&str]| -> Option<usize> {
        names.iter().find_map(|n| {
            headers_lower
                .iter()
                .position(|h| h.contains(n))
        })
    };

    let idx_date = col(&["date"]);
    let idx_start = col(&["sleep start", "sleep_start"]);
    let idx_end = col(&["sleep end", "sleep_end"]);
    let idx_deep = col(&["deep sleep", "deep_sleep"]);
    let idx_light = col(&["light sleep", "light_sleep"]);
    let idx_rem = col(&["rem sleep", "rem_sleep"]);
    let idx_awake = col(&["awake"]);
    let idx_score = col(&["score", "overall"]);
    let idx_total = col(&["total sleep", "total_sleep", "unmeasurable"]);

    let mut rows = Vec::new();

    for result in reader.records() {
        let record = result?;

        let date_str = idx_date
            .and_then(|i| record.get(i))
            .or_else(|| idx_start.and_then(|i| record.get(i)))
            .unwrap_or("")
            .trim()
            .to_string();

        if date_str.is_empty() {
            continue;
        }

        let date = parse_date_from_datetime(&date_str)
            .or_else(|| parse_date(&date_str));
        let Some(date) = date else { continue };

        let deep = idx_deep.and_then(|i| parse_i64(record.get(i)));
        let light = idx_light.and_then(|i| parse_i64(record.get(i)));
        let rem = idx_rem.and_then(|i| parse_i64(record.get(i)));
        let awake = idx_awake.and_then(|i| parse_i64(record.get(i)));

        let total = idx_total
            .and_then(|i| parse_i64(record.get(i)))
            .or_else(|| {
                let sum = deep.unwrap_or(0) + light.unwrap_or(0) + rem.unwrap_or(0);
                if sum > 0 { Some(sum) } else { None }
            });

        rows.push(NewSleepSession {
            date,
            sleep_start: idx_start
                .and_then(|i| record.get(i))
                .and_then(|s| parse_naive_datetime(s)),
            sleep_end: idx_end
                .and_then(|i| record.get(i))
                .and_then(|s| parse_naive_datetime(s)),
            total_sleep_secs: total,
            deep_sleep_secs: deep,
            light_sleep_secs: light,
            rem_sleep_secs: rem,
            awake_secs: awake,
            sleep_score: idx_score.and_then(|i| parse_i64(record.get(i))),
        });
    }

    Ok(rows)
}

// Upsert-style parsers for supplementary data that updates existing rows

pub async fn parse_body_battery(pool: &SqlitePool, data: &str) -> Result<()> {
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(data.as_bytes());

    let headers = reader.headers()?.clone();
    let headers_lower: Vec<String> = headers
        .iter()
        .map(|h| h.to_lowercase().trim().to_string())
        .collect();

    let col = |names: &[&str]| -> Option<usize> {
        names.iter().find_map(|n| {
            headers_lower.iter().position(|h| h.contains(n))
        })
    };

    let idx_date = col(&["date"]);
    let idx_low = col(&["minimum", "min body", "ending"]);
    let idx_high = col(&["maximum", "max body"]);

    for result in reader.records() {
        let record = result?;
        let date_str = idx_date
            .and_then(|i| record.get(i))
            .unwrap_or("")
            .trim();
        let Some(date) = parse_date(date_str) else { continue };

        let low = idx_low.and_then(|i| parse_i64(record.get(i)));
        let high = idx_high.and_then(|i| parse_i64(record.get(i)));

        sqlx::query(
            r#"INSERT INTO daily_wellness (date, body_battery_low, body_battery_high)
               VALUES (?, ?, ?)
               ON CONFLICT(date) DO UPDATE SET
                 body_battery_low = COALESCE(excluded.body_battery_low, body_battery_low),
                 body_battery_high = COALESCE(excluded.body_battery_high, body_battery_high)"#,
        )
        .bind(date.to_string())
        .bind(low)
        .bind(high)
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn parse_hrv(pool: &SqlitePool, data: &str) -> Result<()> {
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(data.as_bytes());

    let headers = reader.headers()?.clone();
    let headers_lower: Vec<String> = headers
        .iter()
        .map(|h| h.to_lowercase().trim().to_string())
        .collect();

    let col = |names: &[&str]| -> Option<usize> {
        names.iter().find_map(|n| {
            headers_lower.iter().position(|h| h.contains(n))
        })
    };

    let idx_date = col(&["date"]);
    let idx_hrv = col(&["night average", "night_average", "hrv", "high frequency"]);

    for result in reader.records() {
        let record = result?;
        let date_str = idx_date
            .and_then(|i| record.get(i))
            .unwrap_or("")
            .trim();
        let Some(date) = parse_date(date_str) else { continue };
        let hrv = idx_hrv.and_then(|i| parse_f64(record.get(i)));

        if hrv.is_none() {
            continue;
        }

        sqlx::query(
            r#"INSERT INTO daily_wellness (date, hrv_night_avg)
               VALUES (?, ?)
               ON CONFLICT(date) DO UPDATE SET
                 hrv_night_avg = COALESCE(excluded.hrv_night_avg, hrv_night_avg)"#,
        )
        .bind(date.to_string())
        .bind(hrv)
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn parse_stress(pool: &SqlitePool, data: &str) -> Result<()> {
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(data.as_bytes());

    let headers = reader.headers()?.clone();
    let headers_lower: Vec<String> = headers
        .iter()
        .map(|h| h.to_lowercase().trim().to_string())
        .collect();

    let col = |names: &[&str]| -> Option<usize> {
        names.iter().find_map(|n| {
            headers_lower.iter().position(|h| h.contains(n))
        })
    };

    let idx_date = col(&["date"]);
    let idx_avg = col(&["average stress", "avg_stress", "stress level"]);

    for result in reader.records() {
        let record = result?;
        let date_str = idx_date
            .and_then(|i| record.get(i))
            .unwrap_or("")
            .trim();
        let Some(date) = parse_date(date_str) else { continue };
        let avg = idx_avg.and_then(|i| parse_i64(record.get(i)));

        if avg.is_none() {
            continue;
        }

        sqlx::query(
            r#"INSERT INTO daily_wellness (date, avg_stress)
               VALUES (?, ?)
               ON CONFLICT(date) DO UPDATE SET
                 avg_stress = COALESCE(excluded.avg_stress, avg_stress)"#,
        )
        .bind(date.to_string())
        .bind(avg)
        .execute(pool)
        .await?;
    }

    Ok(())
}

fn parse_date(s: &str) -> Option<NaiveDate> {
    let s = s.trim();
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .or_else(|_| NaiveDate::parse_from_str(s, "%m/%d/%Y"))
        .or_else(|_| NaiveDate::parse_from_str(s, "%d/%m/%Y"))
        .ok()
}

fn parse_date_from_datetime(s: &str) -> Option<NaiveDate> {
    let s = s.trim();
    // Try to extract the date part from a datetime string
    s.split_whitespace()
        .next()
        .or_else(|| s.split('T').next())
        .and_then(|d| parse_date(d))
}

fn parse_naive_datetime(s: &str) -> Option<chrono::NaiveDateTime> {
    let s = s.trim();
    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
        .or_else(|_| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S"))
        .or_else(|_| chrono::NaiveDateTime::parse_from_str(s, "%m/%d/%Y %H:%M:%S"))
        .ok()
}

fn parse_i64(s: Option<&str>) -> Option<i64> {
    s?.trim()
        .replace(',', "")
        .parse::<f64>()
        .ok()
        .map(|f| f as i64)
}

fn parse_f64(s: Option<&str>) -> Option<f64> {
    s?.trim().replace(',', "").parse::<f64>().ok()
}
