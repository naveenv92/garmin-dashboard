mod api;
mod cli;
mod db;
mod models;
mod parsers;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "garmin-dash", about = "Garmin data dashboard")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Import a Garmin Connect export directory into the database
    Import {
        /// Path to the Garmin export directory
        path: PathBuf,
    },
    /// Start the dashboard web server
    Serve {
        #[arg(long, default_value = "3001")]
        port: u16,
        #[arg(long, default_value = "garmin.db")]
        db: PathBuf,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "garmin_dash=info".into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Import { path } => {
            let pool = db::connect("garmin.db").await?;
            db::migrate(&pool).await?;
            cli::import(&pool, &path).await?;
        }
        Commands::Serve { port, db } => {
            let db_str = db.to_string_lossy();
            let pool = db::connect(&db_str).await?;
            db::migrate(&pool).await?;
            api::serve(pool, port).await?;
        }
    }

    Ok(())
}
