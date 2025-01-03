use dotenv::dotenv;
use envconfig::Envconfig;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Envconfig, Clone)]
pub(crate) struct Config {
    #[envconfig(from = "ADDRESS", default = "0.0.0.0")]
    pub(crate) addr: String,

    #[envconfig(from = "PORT", default = "8080")]
    pub(crate) port: u16,

    #[envconfig(from = "LOG_LEVEL", default = "info")]
    pub(crate) log_level: String,

    #[envconfig(from = "DB_URI", default = "mongodb://localhost:27017")]
    pub(crate) db_uri: String,

    #[envconfig(from = "DB_NAME", default = "uptime-monitor")]
    pub(crate) db_name: String,

    #[envconfig(from = "VERSION", default = "1.0.0")]
    pub(crate) version: String,
}

impl Config {
    pub(crate) fn build() -> Self {
        dotenv().expect("⚠️ failed to load .env file");
        let mut config = Config::init_from_env().expect("⚠️ failed to load configuration");
        let version_parts: Vec<&str> = VERSION.split(".").collect();
        assert_eq!(version_parts.len(), 3, "⚠️ version is malformed");
        config.version = format!("{}.{}", version_parts[0], version_parts[1]);
        config
    }
}
