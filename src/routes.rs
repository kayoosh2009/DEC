use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg
        // Auth
        .route("/auth/register", web::post().to(auth::register))
        .route("/auth/login",    web::post().to(auth::login))
        // Chat
        .route("/chat/send",     web::post().to(chat::send_message))
        // Config
        .route("/api/config",    web::get().to(config_handler::get_config))
        // Memory
        .route("/memory/list",   web::get().to(memory::list))
        .route("/memory/add",    web::post().to(memory::add));
}

// --- Auth handlers ---
mod auth {
    use actix_web::HttpResponse;

    pub async fn register() -> HttpResponse {
        HttpResponse::Ok().body("TODO: register")
    }

    pub async fn login() -> HttpResponse {
        HttpResponse::Ok().body("TODO: login")
    }
}

// --- Chat handlers ---
mod chat {
    use actix_web::HttpResponse;

    pub async fn send_message() -> HttpResponse {
        HttpResponse::Ok().body("TODO: send_message")
    }
}

// --- Memory handlers ---
mod memory {
    use actix_web::HttpResponse;

    pub async fn list() -> HttpResponse {
        HttpResponse::Ok().body("TODO: memory list")
    }

    pub async fn add() -> HttpResponse {
        HttpResponse::Ok().body("TODO: memory add")
    }
}

// --- Config handler ---
mod config_handler {
    use actix_web::{web, HttpResponse};
    use serde_json::json;
    use crate::config::AppConfig;

    pub async fn get_config(config: web::Data<AppConfig>) -> HttpResponse {
        HttpResponse::Ok().json(json!({
            "apiKey":     config.firebase_api_key,
            "authDomain": config.firebase_auth_domain,
            "projectId":  config.firebase_project_id,
            "appId":      config.firebase_app_id,
        }))
    }
}