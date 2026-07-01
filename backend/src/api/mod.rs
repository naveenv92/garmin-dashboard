pub mod activities;
pub mod wellness;

use anyhow::Result;
use axum::{routing::get, Router};
use sqlx::SqlitePool;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

pub async fn serve(pool: SqlitePool, port: u16) -> Result<()> {
    let api = Router::new()
        .route("/api/activities", get(activities::list))
        .route("/api/activities/:id", get(activities::get_one))
        .route("/api/wellness/daily", get(wellness::daily))
        .route("/api/wellness/sleep", get(wellness::sleep_list))
        .route("/api/stats/overview", get(activities::overview))
        .layer(CorsLayer::permissive())
        .with_state(pool);

    // Serve built frontend from ../frontend/dist if it exists
    let frontend_dist = std::path::Path::new("../frontend/dist");
    let app = if frontend_dist.exists() {
        api.fallback_service(ServeDir::new(frontend_dist))
    } else {
        api
    };

    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("Garmin dashboard running at http://localhost:{port}");
    if frontend_dist.exists() {
        println!("  Frontend: http://localhost:{port}");
    } else {
        println!("  Frontend dev server: npm run dev (in frontend/)");
    }
    println!("  API: http://localhost:{port}/api");

    axum::serve(listener, app).await?;
    Ok(())
}
