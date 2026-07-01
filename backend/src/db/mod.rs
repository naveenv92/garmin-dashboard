use anyhow::Result;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

pub async fn connect(db_path: &str) -> Result<SqlitePool> {
    let url = format!("sqlite:{db_path}?mode=rwc");
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await?;
    Ok(pool)
}

pub async fn migrate(pool: &SqlitePool) -> Result<()> {
    // Execute each statement separately (SQLite via sqlx doesn't support multi-statement exec)
    for schema in [
        include_str!("../../migrations/001_initial.sql"),
        include_str!("../../migrations/002_laps.sql"),
    ] {
        for statement in schema.split(';') {
            let stmt = statement.trim();
            if !stmt.is_empty() {
                sqlx::query(stmt).execute(pool).await?;
            }
        }
    }

    // activities predates max_speed/elevation_loss; CREATE TABLE IF NOT EXISTS
    // won't add columns to an already-existing table, so add them explicitly.
    add_column_if_missing(pool, "activities", "max_speed", "REAL").await?;
    add_column_if_missing(pool, "activities", "elevation_loss", "REAL").await?;

    sqlx::query("PRAGMA journal_mode=WAL").execute(pool).await?;
    sqlx::query("PRAGMA foreign_keys=ON").execute(pool).await?;
    Ok(())
}

async fn add_column_if_missing(
    pool: &SqlitePool,
    table: &str,
    column: &str,
    sql_type: &str,
) -> Result<()> {
    let exists: Option<(String,)> = sqlx::query_as(&format!(
        "SELECT name FROM pragma_table_info('{table}') WHERE name = '{column}'"
    ))
    .fetch_optional(pool)
    .await?;

    if exists.is_none() {
        sqlx::query(&format!("ALTER TABLE {table} ADD COLUMN {column} {sql_type}"))
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn file_already_imported(pool: &SqlitePool, hash: &str) -> Result<bool> {
    let row: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM imported_files WHERE file_hash = ?")
            .bind(hash)
            .fetch_optional(pool)
            .await?;
    Ok(row.is_some())
}

pub async fn record_imported_file(
    pool: &SqlitePool,
    file_path: &str,
    hash: &str,
) -> Result<()> {
    sqlx::query(
        "INSERT OR IGNORE INTO imported_files (file_path, file_hash) VALUES (?, ?)",
    )
    .bind(file_path)
    .bind(hash)
    .execute(pool)
    .await?;
    Ok(())
}
