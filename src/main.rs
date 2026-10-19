#![allow(dead_code, unused_imports)]

mod ai;
mod config;
mod database;
mod routes;
mod telegram;

use actix_files::Files;
use actix_web::{web, App, HttpServer};
use config::AppConfig;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let config = AppConfig::from_env().expect("Failed to load config");
    let client = reqwest::Client::new();

    let host = config.host.clone();
    let port = config.port.clone();

    println!("DEC запущен: http://{}:{}", host, port);

    let startup_msg = format!("🚀 <b>DEC Сервер запущен</b>\nАдрес: <code>http://{}:{}</code>", host, port);
    telegram::send_log(&client, &config.telegram_bot_token, &config.telegram_channel, &startup_msg).await;

    let config = web::Data::new(config);
    let client = web::Data::new(client);

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
