use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg
        // Auth
        .route("/auth/register", web::post().to(auth::register))
        .route("/auth/login",    web::post().to(auth::login))
        // Chat
        .route("/chat/send",     web::post().to(chat::send_message))
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