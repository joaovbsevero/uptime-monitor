mod api;
mod config;
mod dependencies;
mod middlewares;
mod models;
mod monitor;

use poem::{handler, listener::TcpListener, middleware::AddData, EndpointExt, Route};
use poem_openapi::OpenApiService;

use api::MonitorAPI;
use config::Config;
use tokio::{fs::File, io::AsyncReadExt};

#[handler]
async fn favicon_handler() -> Vec<u8> {
    let favicon = File::open("resources/icons/favicon.png").await;
    match favicon {
        Ok(mut file) => {
            let mut buffer = vec![];
            match file.read_to_end(&mut buffer).await {
                Ok(_) => buffer,
                Err(_) => vec![],
            }
        }
        Err(_) => return vec![],
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Load .env vars
    let config = Config::build();

    // Init dependencies
    dependencies::log(&config);
    let db = dependencies::db(&config).await;

    // Spawn monitor process
    tokio::spawn(monitor::start(db.clone()));

    // Setup service
    let api_service = OpenApiService::new(MonitorAPI, "Uptime Monitor 📢 ", config.version);
    let swagger = api_service.swagger_ui();
    let redoc = api_service.redoc();
    let app = Route::new()
        .nest("/favicon.ico", favicon_handler)
        .nest("/", api_service)
        .nest("/docs", swagger)
        .nest("/redoc", redoc)
        .around(middlewares::log)
        .with(AddData::new(db));

    // Start server
    let address = format!("{}:{}", config.addr, config.port);
    poem::Server::new(TcpListener::bind(address)).run(app).await
}
