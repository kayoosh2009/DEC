mod ai;
mod db;
mod routes;

use actix_web::{App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    HttpServer::new(|| {
        App::new()
            // routes will be added here
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}