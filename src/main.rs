#![allow(dead_code, unused_imports)]

mod ai;
mod config;
mod database;
mod routes;

use actix_files::Files;
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

    println!("DEC запущен: http://{}:{}", host, port);

    HttpServer::new(move || {
        App::new()
            .app_data(config.clone())
            .app_data(client.clone())
            .configure(routes::init)
            .service(Files::new("/", "static").index_file("index.html"))
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}
