use crate::ai::{MemoryPoint, Message};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

const FIRESTORE_BASE: &str = "https://firestore.googleapis.com/v1";
const FIREBASE_AUTH_BASE: &str = "https://identitytoolkit.googleapis.com/v1";

// --- Auth ---

#[derive(Deserialize)]
pub struct AuthResponse {
    pub id_token: String,
    pub local_id: String, // user uid
}

/// Sign in with email + password, returns id_token and uid
pub async fn sign_in(
    client: &reqwest::Client,
    api_key: &str,
    email: &str,
    password: &str,
) -> anyhow::Result<AuthResponse> {
    let url = format!("{}/accounts:signInWithPassword?key={}", FIREBASE_AUTH_BASE, api_key);

    let resp = client
        .post(&url)
        .json(&json!({
            "email": email,
            "password": password,
            "returnSecureToken": true
        }))
        .send()
        .await?
        .json::<AuthResponse>()
        .await?;

    Ok(resp)
}

/// Register new user with email + password
pub async fn sign_up(
    client: &reqwest::Client,
    api_key: &str,
    email: &str,
    password: &str,
) -> anyhow::Result<AuthResponse> {
    let url = format!("{}/accounts:signUp?key={}", FIREBASE_AUTH_BASE, api_key);

    let resp = client
        .post(&url)
        .json(&json!({
            "email": email,
            "password": password,
            "returnSecureToken": true
        }))
        .send()
        .await?
        .json::<AuthResponse>()
        .await?;

    Ok(resp)
}

// --- Firestore helpers ---

/// Extracts a string field from a Firestore document value
fn get_string_field(doc: &Value, field: &str) -> Option<String> {
    doc["fields"][field]["stringValue"]
        .as_str()
        .map(|s| s.to_string())
}

// --- Messages (last 5 per chat) ---

/// Save a message to Firestore: users/{uid}/chats/{chat_id}/messages/{msg_id}
pub async fn save_message(
    client: &reqwest::Client,
    project_id: &str,
    id_token: &str,
    uid: &str,
    chat_id: &str,
    msg_id: &str,
    message: &Message,
    timestamp: &str,
) -> anyhow::Result<()> {
    let url = format!(
        "{}/projects/{}/databases/(default)/documents/users/{}/chats/{}/messages/{}",
        FIRESTORE_BASE, project_id, uid, chat_id, msg_id
    );

    client
        .patch(&url)
        .bearer_auth(id_token)
        .json(&json!({
            "fields": {
                "role":      { "stringValue": message.role },
                "content":   { "stringValue": message.content },
                "timestamp": { "stringValue": timestamp }
            }
        }))
        .send()
        .await?;

    Ok(())
}

/// Load last 5 messages from a chat
pub async fn load_recent_messages(
    client: &reqwest::Client,
    project_id: &str,
    id_token: &str,
    uid: &str,
    chat_id: &str,
) -> anyhow::Result<Vec<Message>> {
    let url = format!(
        "{}/projects/{}/databases/(default)/documents/users/{}/chats/{}/messages",
        FIRESTORE_BASE, project_id, uid, chat_id
    );

    let resp: Value = client
        .get(&url)
        .bearer_auth(id_token)
        .send()
        .await?
        .json()
        .await?;

    let mut messages: Vec<(String, Message)> = vec![];

    if let Some(docs) = resp["documents"].as_array() {
        for doc in docs {
            let role = get_string_field(doc, "role").unwrap_or_default();
            let content = get_string_field(doc, "content").unwrap_or_default();
            let timestamp = get_string_field(doc, "timestamp").unwrap_or_default();

            messages.push((timestamp, Message { role, content }));
        }
    }

    // Sort by timestamp, keep last 5
    messages.sort_by(|a, b| a.0.cmp(&b.0));
    let recent: Vec<Message> = messages.into_iter().rev().take(5).rev().map(|(_, m)| m).collect();

    Ok(recent)
}

// --- Memory points (diary) ---

#[derive(Serialize, Deserialize)]
pub struct MemoryPointRecord {
    pub id: String,
    pub summary: String,
    pub timestamp: String,
}

/// Save a memory point to Firestore: users/{uid}/memory/{point_id}
pub async fn save_memory_point(
    client: &reqwest::Client,
    project_id: &str,
    id_token: &str,
    uid: &str,
    point_id: &str,
    summary: &str,
    timestamp: &str,
) -> anyhow::Result<()> {
    let url = format!(
        "{}/projects/{}/databases/(default)/documents/users/{}/memory/{}",
        FIRESTORE_BASE, project_id, uid, point_id
    );

    client
        .patch(&url)
        .bearer_auth(id_token)
        .json(&json!({
            "fields": {
                "summary":   { "stringValue": summary },
                "timestamp": { "stringValue": timestamp }
            }
        }))
        .send()
        .await?;

    Ok(())
}

/// Load all memory points for a user
pub async fn load_memory_points(
    client: &reqwest::Client,
    project_id: &str,
    id_token: &str,
    uid: &str,
) -> anyhow::Result<Vec<MemoryPoint>> {
    let url = format!(
        "{}/projects/{}/databases/(default)/documents/users/{}/memory",
        FIRESTORE_BASE, project_id, uid
    );

    let resp: Value = client
        .get(&url)
        .bearer_auth(id_token)
        .send()
        .await?
        .json()
        .await?;

    let mut points: Vec<MemoryPoint> = vec![];

    if let Some(docs) = resp["documents"].as_array() {
        for doc in docs {
            let summary = get_string_field(doc, "summary").unwrap_or_default();
            let timestamp = get_string_field(doc, "timestamp").unwrap_or_default();
            points.push(MemoryPoint { summary, timestamp });
        }
    }

    // Sort by timestamp ascending
    points.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    Ok(points)
}