use std::env;

#[derive(Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: String,
    pub ollama_url: String,
    pub ollama_api_key: String,
    pub ollama_model: String,
    pub firebase_project_id: String,
    pub firebase_api_key: String,
    pub firebase_auth_domain: String,
    pub firebase_app_id: String,
    pub telegram_bot_token: String,
    pub telegram_channel: String,
    pub firebase_storage_bucket: String,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let project_id = env::var("FIREBASE_PROJECT_ID")?;
        let storage_bucket = env::var("FIREBASE_STORAGE_BUCKET")
            .unwrap_or_else(|_| format!("{}.firebasestorage.app", project_id));

        Ok(Self {
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT").unwrap_or_else(|_| "8080".to_string()),
            ollama_url: env::var("OLLAMA_URL")?,
            ollama_api_key: env::var("OLLAMA_API_KEY")?,
            ollama_model: env::var("OLLAMA_MODEL")?,
            firebase_project_id: project_id,
            firebase_api_key: env::var("FIREBASE_API_KEY")?,
            firebase_auth_domain: env::var("FIREBASE_AUTH_DOMAIN")?,
            firebase_app_id: env::var("FIREBASE_APP_ID")?,
            telegram_bot_token: env::var("TELEGRAM_BOT_TOKEN").unwrap_or_default(),
            telegram_channel: env::var("TELEGRAM_CHANNEL").unwrap_or_else(|_| "@openlogs_kayoosh".to_string()),
            firebase_storage_bucket: storage_bucket,
        })
    }
}
