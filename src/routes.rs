use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg
        // Auth
        .route("/auth/register", web::post().to(auth::register))
        .route("/auth/login", web::post().to(auth::login))
        // Chat
        .route("/chat/send", web::post().to(chat::send_message))
        .route("/chat/star", web::post().to(chat::star_message))
        .route("/chat/list", web::get().to(chat::list_chats))
        .route("/chat/save", web::post().to(chat::save_chat))
        .route("/chat/delete", web::post().to(chat::delete_chat))
        .route("/chat/messages", web::get().to(chat::get_messages))
        .route("/chat/export", web::get().to(chat::export_chats))
        // Config
        .route("/api/config", web::get().to(config_handler::get_config))
        // Dashboard
        .route("/dashboard/stats", web::get().to(dashboard::platform_stats))
        .route("/dashboard/user", web::get().to(dashboard::user_stats))
        // Memory
        .route("/memory/list", web::get().to(memory::list))
        .route("/memory/clear", web::post().to(memory::clear))
        .route("/memory/add", web::post().to(memory::add));
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
    use crate::ai::{self, Message};
    use crate::config::AppConfig;
    use crate::database::{memory, messages};
    use crate::telegram;
    use actix_web::{web, HttpResponse};
    use serde::{Deserialize, Serialize};

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

    #[derive(Deserialize)]
    pub struct SaveChatRequest {
        pub chat_id: String,
        pub title: String,
        pub timestamp: String,
        pub pinned: Option<bool>,
        pub id_token: String,
        pub uid: String,
    }

    pub async fn get_messages(
        query: web::Query<std::collections::HashMap<String, String>>,
        config: web::Data<AppConfig>,
        client: web::Data<reqwest::Client>,
    ) -> HttpResponse {
        let uid = match query.get("uid") {
            Some(v) => v,
            None => return HttpResponse::BadRequest().body("missing uid"),
        };
        let id_token = match query.get("id_token") {
            Some(v) => v,
            None => return HttpResponse::BadRequest().body("missing id_token"),
        };
        let chat_id = match query.get("chat_id") {
            Some(v) => v,
            None => return HttpResponse::BadRequest().body("missing chat_id"),
        };

        let url = format!(
            "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/users/{}/chats/{}/messages",
            config.firebase_project_id, uid, chat_id
        );

        let resp = client.get(&url).bearer_auth(id_token).send().await;

        match resp {
            Ok(r) => {
                let data: serde_json::Value = r.json().await.unwrap_or_default();
                let mut msgs: Vec<serde_json::Value> = data["documents"]
                    .as_array()
                    .unwrap_or(&vec![])
                    .iter()
                    .map(|doc| serde_json::json!({
                        "role":      doc["fields"]["role"]["stringValue"].as_str().unwrap_or(""),
                        "content":   doc["fields"]["content"]["stringValue"].as_str().unwrap_or(""),
                        "timestamp": doc["fields"]["timestamp"]["stringValue"].as_str().unwrap_or(""),
                        "starred":   doc["fields"]["starred"]["booleanValue"].as_bool().unwrap_or(false)
                    }))
                    .collect();

                msgs.sort_by(|a, b| {
                    a["timestamp"]
                        .as_str()
                        .unwrap_or("")
                        .cmp(b["timestamp"].as_str().unwrap_or(""))
                });

                HttpResponse::Ok().json(msgs)
            }
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        }
    }

    pub async fn save_chat(
        body: web::Json<SaveChatRequest>,
        config: web::Data<AppConfig>,
        client: web::Data<reqwest::Client>,
    ) -> HttpResponse {
        let url = format!(
            "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/users/{}/chats/{}",
            config.firebase_project_id, body.uid, body.chat_id
        );

        let result = client
            .patch(&url)
            .bearer_auth(&body.id_token)
            .json(&serde_json::json!({
                "fields": {
                    "title":     { "stringValue": body.title },
                    "timestamp": { "stringValue": body.timestamp },
                    "pinned":    { "booleanValue": body.pinned.unwrap_or(false) }
                }
            }))
            .send()
            .await;

        match result {
            Ok(r) => {
                let log_text = format!(
                    "💬 <b>Новый чат</b>\n<b>ID:</b> <code>{}</code>\n<b>Название:</b> {}\n<b>Пользователь (UID):</b> <code>{}</code>",
                    body.chat_id, body.title, body.uid
                );
                telegram::send_log(&client, &config.telegram_bot_token, &config.telegram_channel, &log_text).await;
                HttpResponse::Ok().body("ok")
            }
            Err(e) => {
                println!("[save_chat] error: {}", e);
                HttpResponse::InternalServerError().body(e.to_string())
            }
        }
    }

    #[derive(Deserialize)]
    pub struct DeleteChatRequest {
        pub chat_id: String,
        pub id_token: String,
        pub uid: String,
        pub keep_for_training: Option<bool>,
    }

    pub async fn delete_chat(
        body: web::Json<DeleteChatRequest>,
        config: web::Data<AppConfig>,
        client: web::Data<reqwest::Client>,
    ) -> HttpResponse {
        let keep = body.keep_for_training.unwrap_or(false);

        if keep {
            let log_msg = format!(
                "🧠 <b>Чат архивирован для обучения</b>\n<b>Chat ID:</b> <code>{}</code>\n<b>User UID:</b> <code>{}</code>",
                body.chat_id, body.uid
            );
            crate::telegram::send_log(&client, &config.telegram_bot_token, &config.telegram_channel, &log_msg).await;
        }

        // Удаляем метаданные чата из списка пользователя
        let url = format!(
            "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/users/{}/chats/{}",
            config.firebase_project_id, body.uid, body.chat_id
        );

        let result = client.delete(&url).bearer_auth(&body.id_token).send().await;

        match result {
            Ok(_) => HttpResponse::Ok().body("ok"),
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        }
    }

    pub async fn list_chats(
        query: web::Query<std::collections::HashMap<String, String>>,
        config: web::Data<AppConfig>,
        client: web::Data<reqwest::Client>,
    ) -> HttpResponse {
        let uid = match query.get("uid") {
            Some(v) => v,
            None => return HttpResponse::BadRequest().body("missing uid"),
        };
        let id_token = match query.get("id_token") {
            Some(v) => v,
            None => return HttpResponse::BadRequest().body("missing id_token"),
        };

        let url = format!(
            "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/users/{}/chats",
            config.firebase_project_id, uid
        );

        let resp = client.get(&url).bearer_auth(id_token).send().await;

        match resp {
            Ok(r) => {
                let data: serde_json::Value = r.json().await.unwrap_or_default();
                let chats: Vec<serde_json::Value> = data["documents"]
                    .as_array()
                    .unwrap_or(&vec![])
                    .iter()
                    .map(|doc| {
                        let name = doc["name"].as_str().unwrap_or("");
                        let chat_id = name.split('/').last().unwrap_or("");
                        serde_json::json!({
                            "chat_id":   chat_id,
                            "title":     doc["fields"]["title"]["stringValue"].as_str().unwrap_or("Untitled"),
                            "timestamp": doc["fields"]["timestamp"]["stringValue"].as_str().unwrap_or(""),
                            "pinned":    doc["fields"]["pinned"]["booleanValue"].as_bool().unwrap_or(false)
                        })
                    })
                    .collect();

                HttpResponse::Ok().json(chats)
            }
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        }
    }

    pub async fn export_chats(
        query: web::Query<std::collections::HashMap<String, String>>,
        config: web::Data<AppConfig>,
        client: web::Data<reqwest::Client>,
    ) -> HttpResponse {
        let uid = match query.get("uid") { Some(v) => v, None => return HttpResponse::BadRequest().body("missing uid") };
        let id_token = match query.get("id_token") { Some(v) => v, None => return HttpResponse::BadRequest().body("missing id_token") };

        // 1. Получаем все чаты
        let chats_url = format!(
            "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/users/{}/chats",
            config.firebase_project_id, uid
        );

        let chats_resp = client.get(&chats_url).bearer_auth(id_token).send().await;
        let mut export_dataset = Vec::new();

        if let Ok(r) = chats_resp {
            let data: serde_json::Value = r.json().await.unwrap_or_default();
            if let Some(docs) = data["documents"].as_array() {
                for doc in docs {
                    let name = doc["name"].as_str().unwrap_or("");
                    let chat_id = name.split('/').last().unwrap_or("");
                    let title = doc["fields"]["title"]["stringValue"].as_str().unwrap_or("Untitled");

                    // 2. Для каждого чата получаем сообщения
                    let msgs_url = format!(
                        "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/users/{}/chats/{}/messages",
                        config.firebase_project_id, uid, chat_id
                    );

                    if let Ok(mr) = client.get(&msgs_url).bearer_auth(id_token).send().await {
                        let mdata: serde_json::Value = mr.json().await.unwrap_or_default();
                        if let Some(mdocs) = mdata["documents"].as_array() {
                            let mut messages_list = Vec::new();
                            for mdoc in mdocs {
                                let role = mdoc["fields"]["role"]["stringValue"].as_str().unwrap_or("");
                                let content = mdoc["fields"]["content"]["stringValue"].as_str().unwrap_or("");
                                let ts = mdoc["fields"]["timestamp"]["stringValue"].as_str().unwrap_or("");
                                messages_list.push((ts, serde_json::json!({
                                    "role": role,
                                    "content": content
                                })));
                            }
                            messages_list.sort_by(|a, b| a.0.cmp(&b.0));
                            let clean_messages: Vec<serde_json::Value> = messages_list.into_iter().map(|(_, m)| m).collect();

                            if !clean_messages.is_empty() {
                                export_dataset.push(serde_json::json!({
                                    "title": title,
                                    "dialog": clean_messages
                                }));
                            }
                        }
                    }
                }
            }
        }

        HttpResponse::Ok().json(export_dataset)
    }

    #[derive(Deserialize)]
    pub struct StarRequest {
        pub chat_id: String,
        pub msg_id: String,
        pub message: Message,
        pub timestamp: String,
        pub id_token: String,
        pub uid: String,
    }

    pub async fn star_message(
        body: web::Json<StarRequest>,
        config: web::Data<AppConfig>,
        client: web::Data<reqwest::Client>,
    ) -> HttpResponse {
        let result = messages::save(
            &client,
            &config.firebase_project_id,
            &body.id_token,
            &body.uid,
            &body.chat_id,
            &body.msg_id,
            &body.message,
            &body.timestamp,
            true,
            0,
        )
        .await;

        match result {
            Ok(_) => HttpResponse::Ok().body("ok"),
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        }
    }

    pub async fn send_message(
        body: web::Json<SendRequest>,
        config: web::Data<AppConfig>,
        client: web::Data<reqwest::Client>,
    ) -> HttpResponse {
        // Если это запрос на заголовок — генерируем ответ без сохранения в историю и БД
        if body.message.starts_with("Give this chat a short title") {
            let ai_response = ai::generate_response(
                &client,
                &config.ollama_url,
                &config.ollama_api_key,
                &config.ollama_model,
                &body.message,
                &body.timestamp,
                vec![],
                vec![],
                vec![],
            )
            .await;

            return match ai_response {
                Ok(text) => HttpResponse::Ok().json(SendResponse {
                    message: text,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                }),
                Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            };
        }

        // 1. Грузим последние 20 сообщений
        let recent = messages::load_recent(
            &client,
            &config.firebase_project_id,
            &body.id_token,
            &body.uid,
            &body.chat_id,
            20,
        )
        .await
        .unwrap_or_default();

        // 2. Грузим избранные сообщения
        let starred = messages::load_starred(
            &client,
            &config.firebase_project_id,
            &body.id_token,
            &body.uid,
            &body.chat_id,
        )
        .await
        .unwrap_or_default();

        // 3. Грузим memory points
        let memory_points = memory::load_all(
            &client,
            &config.firebase_project_id,
            &body.id_token,
            &body.uid,
        )
        .await
        .unwrap_or_default();

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
        )
        .await;

        match ai_response {
            Ok(text) => {
                // 6. Считаем токены (примерно: 1 токен ~ 4 символа)
                let tokens = ((body.message.len() + text.len()) / 4) as u64;

                // 7. Сохраняем сообщение пользователя
                let msg_id = uuid::Uuid::new_v4().to_string();
                let user_msg = Message {
                    role: "user".to_string(),
                    content: body.message.clone(),
                };
                let _ = messages::save(
                    &client,
                    &config.firebase_project_id,
                    &body.id_token,
                    &body.uid,
                    &body.chat_id,
                    &msg_id,
                    &user_msg,
                    &body.timestamp,
                    false,
                    tokens / 2,
                )
                .await;

                // 8. Сохраняем ответ ИИ
                let ai_msg = Message {
                    role: "assistant".to_string(),
                    content: text.clone(),
                };
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
                    tokens / 2,
                )
                .await;

                // 9. Обновляем глобальную статистику
                let _ = messages::increment_platform_stats(
                    &client,
                    &config.firebase_project_id,
                    &body.id_token,
                    tokens,
                )
                .await;

                // 10. Логируем в Telegram (активный пользователь, id сообщения и вес)
                let log_text = format!(
                    "✉️ <b>Сообщение обработано</b>\n<b>ID:</b> <code>{}</code>\n<b>Вес:</b> {} токенов\n<b>UID:</b> <code>{}</code>",
                    msg_id, tokens, body.uid
                );
                telegram::send_log(&client, &config.telegram_bot_token, &config.telegram_channel, &log_text).await;

                // 11. Фоновое извлечение фактов о пользователе в память
                let bg_client = client.get_ref().clone();
                let bg_config = config.get_ref().clone();
                let bg_uid = body.uid.clone();
                let bg_id_token = body.id_token.clone();
                let bg_user_msg = body.message.clone();

                actix_web::rt::spawn(async move {
                    let prompt = format!(
                        "Проанализируй фразу пользователя: \"{}\". Содержит ли она личные факты о самом пользователе (его имя, увлечения, предпочтения, работа, привычки)? Если ДА, сформулируй один краткий факт от третьего лица (например: 'Пользователя зовут Андрей' или 'Пользователь любит пиццу'). Если личных фактов нет — ответь строго одним словом: НЕТ",
                        bg_user_msg
                    );

                    let req = serde_json::json!({
                        "model": bg_config.ollama_model,
                        "messages": [{"role": "user", "content": prompt}],
                        "stream": false
                    });

                    if let Ok(resp) = bg_client
                        .post(format!("{}/api/chat", bg_config.ollama_url))
                        .header("Authorization", format!("Bearer {}", bg_config.ollama_api_key))
                        .header("Content-Type", "application/json")
                        .json(&req)
                        .send()
                        .await
                    {
                        if let Ok(data) = resp.json::<serde_json::Value>().await {
                            let fact = data["message"]["content"].as_str().unwrap_or("").trim().to_string();
                            if !fact.is_empty() && !fact.to_uppercase().contains("НЕТ") && fact.len() < 120 {
                                let point_id = uuid::Uuid::new_v4().to_string();
                                let now = chrono::Utc::now().to_rfc3339();
                                let _ = memory::save(
                                    &bg_client,
                                    &bg_config.firebase_project_id,
                                    &bg_id_token,
                                    &bg_uid,
                                    &point_id,
                                    &fact,
                                    &now,
                                ).await;
                            }
                        }
                    }
                });

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
    use actix_web::{web, HttpResponse};
    use std::collections::HashMap;
    use crate::config::AppConfig;
    use crate::database::memory as db_memory;

    pub async fn list(
        query: web::Query<HashMap<String, String>>,
        config: web::Data<AppConfig>,
        client: web::Data<reqwest::Client>,
    ) -> HttpResponse {
        let uid = match query.get("uid") {
            Some(v) => v,
            None => return HttpResponse::BadRequest().body("missing uid"),
        };
        let id_token = match query.get("id_token") {
            Some(v) => v,
            None => return HttpResponse::BadRequest().body("missing id_token"),
        };

        match db_memory::load_all(&client, &config.firebase_project_id, id_token, uid).await {
            Ok(points) => HttpResponse::Ok().json(points),
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        }
    }

    #[derive(serde::Deserialize)]
    pub struct ClearRequest {
        pub uid: String,
        pub id_token: String,
    }

    pub async fn clear(
        body: web::Json<ClearRequest>,
        config: web::Data<AppConfig>,
        client: web::Data<reqwest::Client>,
    ) -> HttpResponse {
        let url = format!(
            "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/users/{}/memory",
            config.firebase_project_id, body.uid
        );

        let resp = client.get(&url).bearer_auth(&body.id_token).send().await;
        if let Ok(r) = resp {
            let data: serde_json::Value = r.json().await.unwrap_or_default();
            if let Some(docs) = data["documents"].as_array() {
                for doc in docs {
                    if let Some(name) = doc["name"].as_str() {
                        let del_url = format!("https://firestore.googleapis.com/v1/{}", name);
                        let _ = client.delete(&del_url).bearer_auth(&body.id_token).send().await;
                    }
                }
            }
        }

        HttpResponse::Ok().body("ok")
    }

    pub async fn add() -> HttpResponse {
        HttpResponse::Ok().body("ok")
    }
}

// --- Dashboard handlers ---
mod dashboard {
    use crate::config::AppConfig;
    use actix_web::{web, HttpResponse};
    use std::collections::HashMap;

    pub async fn platform_stats(
        query: web::Query<HashMap<String, String>>,
        config: web::Data<AppConfig>,
        client: web::Data<reqwest::Client>,
    ) -> HttpResponse {
        let id_token = match query.get("id_token") {
            Some(v) => v,
            None => return HttpResponse::BadRequest().body("missing id_token"),
        };

        let url = format!(
            "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/platform/stats",
            config.firebase_project_id
        );

        let resp = client.get(&url).bearer_auth(id_token).send().await;

        match resp {
            Ok(r) => {
                let data: serde_json::Value = r.json().await.unwrap_or_default();
                let tokens = data["fields"]["tokens_used"]["integerValue"]
                    .as_str()
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(0);
                let messages = data["fields"]["messages_count"]["integerValue"]
                    .as_str()
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(0);

                HttpResponse::Ok().json(serde_json::json!({
                    "tokens_used":    tokens,
                    "messages_count": messages
                }))
            }
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        }
    }

    pub async fn user_stats(
        query: web::Query<HashMap<String, String>>,
        config: web::Data<AppConfig>,
        client: web::Data<reqwest::Client>,
    ) -> HttpResponse {
        let uid = match query.get("uid") {
            Some(v) => v,
            None => return HttpResponse::BadRequest().body("missing uid"),
        };
        let id_token = match query.get("id_token") {
            Some(v) => v,
            None => return HttpResponse::BadRequest().body("missing id_token"),
        };

        // Загружаем все чаты пользователя
        let chats_url = format!(
            "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/users/{}/chats",
            config.firebase_project_id, uid
        );

        let chats_resp: serde_json::Value = client
            .get(&chats_url)
            .bearer_auth(id_token)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap_or_default();

        let now = chrono::Utc::now();
        let month_prefix = now.format("%Y-%m").to_string();
        let mut tokens_this_month: u64 = 0;
        let mut daily_tokens: HashMap<String, u64> = HashMap::new();

        // Для каждого чата грузим сообщения
        if let Some(chats) = chats_resp["documents"].as_array() {
            for chat in chats {
                let chat_name = chat["name"].as_str().unwrap_or("");
                let chat_id = chat_name.split('/').last().unwrap_or("");

                let msgs_url = format!(
                    "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/users/{}/chats/{}/messages",
                    config.firebase_project_id, uid, chat_id
                );

                let msgs_resp: serde_json::Value = client
                    .get(&msgs_url)
                    .bearer_auth(id_token)
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap_or_default();

                if let Some(msgs) = msgs_resp["documents"].as_array() {
                    for msg in msgs {
                        let ts = msg["fields"]["timestamp"]["stringValue"]
                            .as_str()
                            .unwrap_or("");
                        let tokens = msg["fields"]["tokens"]["integerValue"]
                            .as_str()
                            .and_then(|v| v.parse::<u64>().ok())
                            .unwrap_or(0);

                        if ts.starts_with(&month_prefix) {
                            tokens_this_month += tokens;
                            let day = &ts[..10]; // "YYYY-MM-DD"
                            *daily_tokens.entry(day.to_string()).or_insert(0) += tokens;
                        }
                    }
                }
            }
        }

        HttpResponse::Ok().json(serde_json::json!({
            "tokens_this_month": tokens_this_month,
            "daily_tokens":      daily_tokens
        }))
    }
}

// --- Config handler ---
mod config_handler {
    use crate::config::AppConfig;
    use actix_web::{web, HttpResponse};
    use serde_json::json;

    pub async fn get_config(config: web::Data<AppConfig>) -> HttpResponse {
        HttpResponse::Ok().json(json!({
            "apiKey":     config.firebase_api_key,
            "authDomain": config.firebase_auth_domain,
            "projectId":  config.firebase_project_id,
            "appId":      config.firebase_app_id,
        }))
    }
}
