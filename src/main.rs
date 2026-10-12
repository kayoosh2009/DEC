mod ai;
mod config;
mod database;
mod routes;

use actix_web::{web, App, HttpServer};
use config::AppConfig;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let config = AppConfig::from_env().expect("Failed to load config");
    let client = reqwest::Client::new();

    let config = web::Data::new(config);
    let client = web::Data::new(client);

    let host = config.host.clone();
    let port = config.port.clone();

    HttpServer::new(move || {
        App::new()
            .app_data(config.clone())
            .app_data(client.clone())
            .configure(routes::init)
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}