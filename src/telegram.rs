use reqwest::Client;
use serde_json::json;

pub async fn send_log(client: &Client, bot_token: &str, channel: &str, text: &str) {
    if bot_token.is_empty() || channel.is_empty() {
        return;
    }

    let url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);

    let _ = client
        .post(&url)
        .json(&json!({
            "chat_id": channel,
            "text": text,
            "parse_mode": "HTML"
        }))
        .send()
        .await;
}