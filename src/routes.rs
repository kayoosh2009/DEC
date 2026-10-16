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
    use actix_web::{web, HttpResponse};
    use serde::{Deserialize, Serialize};
    use crate::config::AppConfig;
    use crate::ai::{self, Message};
    use crate::database::{messages, memory};

    #[derive(Deserialize)]
    pub struct SendRequest {
        pub chat_id: String,
        pub message: String,
        pub timestamp: String,
        pub id_token: String,
        pub uid: String,
    }

    #[derive(Serialize)]
    pub struct SendResponse {
        pub message: String,
        pub timestamp: String,
    }

    pub async fn send_message(
        body: web::Json<SendRequest>,
        config: web::Data<AppConfig>,
        client: web::Data<reqwest::Client>,
    ) -> HttpResponse {
        // 1. Грузим последние 20 сообщений
        let recent = messages::load_recent(
            &client,
            &config.firebase_project_id,
            &body.id_token,
            &body.uid,
            &body.chat_id,
            20,
        ).await.unwrap_or_default();

        // 2. Грузим избранные сообщения
        let starred = messages::load_starred(
            &client,
            &config.firebase_project_id,
            &body.id_token,
            &body.uid,
            &body.chat_id,
        ).await.unwrap_or_default();

        // 3. Грузим memory points
        let memory_points = memory::load_all(
            &client,
            &config.firebase_project_id,
            &body.id_token,
            &body.uid,
        ).await.unwrap_or_default();

        // 4. Генерируем ответ
        let ai_response = ai::generate_response(
            &client,
            &config.ollama_url,
            &config.ollama_api_key,
            &config.ollama_model,
            &body.message,
            &body.timestamp,
            recent,
            starred,
            memory_points,
        ).await;

        match ai_response {
            Ok(text) => {
                // 5. Сохраняем сообщение пользователя
                let user_msg = Message { role: "user".to_string(), content: body.message.clone() };
                let _ = messages::save(
                    &client,
                    &config.firebase_project_id,
                    &body.id_token,
                    &body.uid,
                    &body.chat_id,
                    &uuid::Uuid::new_v4().to_string(),
                    &user_msg,
                    &body.timestamp,
                    false,
                ).await;

                // 6. Сохраняем ответ ИИ
                let ai_msg = Message { role: "assistant".to_string(), content: text.clone() };
                let ai_timestamp = chrono::Utc::now().to_rfc3339();
                let _ = messages::save(
                    &client,
                    &config.firebase_project_id,
                    &body.id_token,
                    &body.uid,
                    &body.chat_id,
                    &uuid::Uuid::new_v4().to_string(),
                    &ai_msg,
                    &ai_timestamp,
                    false,
                ).await;

                HttpResponse::Ok().json(SendResponse {
                    message: text,
                    timestamp: ai_timestamp,
                })
            }
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        }
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