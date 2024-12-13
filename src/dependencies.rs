use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use crate::config::Config;
use mongodb::{Client, Database};

pub(crate) fn log(config: &Config) {
    let level_filter = match config.log_level.as_str() {
        "error" => LevelFilter::ERROR,
        "warn" => LevelFilter::WARN,
        "info" => LevelFilter::INFO,
        "debug" => LevelFilter::DEBUG,
        "trace" => LevelFilter::TRACE,
        _ => LevelFilter::INFO,
    };
    let filter = EnvFilter::builder()
        .with_default_directive(level_filter.into())
        .from_env_lossy();

    tracing_subscriber::registry()
        .with(fmt::layer().with_filter(filter))
        .init();
}

pub(crate) async fn db(config: &Config) -> Database {
    let client = Client::with_uri_str(&config.db_uri)
        .await
        .expect(&format!("Invalid connection URI: {}", config.db_uri));
    let db = client.database(&config.db_name);


    db
}
