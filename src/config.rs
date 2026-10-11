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
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT").unwrap_or_else(|_| "8080".to_string()),
            ollama_url: env::var("OLLAMA_URL")?,
            ollama_api_key: env::var("OLLAMA_API_KEY")?,
            ollama_model: env::var("OLLAMA_MODEL")?,
            firebase_project_id: env::var("FIREBASE_PROJECT_ID")?,
            firebase_api_key: env::var("FIREBASE_API_KEY")?,
        })
    }
}