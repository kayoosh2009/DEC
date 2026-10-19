//! Database layer — Firebase Firestore + Auth
//!
//! Structure:
//!   auth::*    — sign_in, sign_up
//!   messages::* — save_message, load_recent
//!   memory::*  — save_point, load_all

use anyhow::anyhow;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::ai::{MemoryPoint, Message};

// ─── Constants ───────────────────────────────────────────────────────────────

const FIRESTORE_BASE: &str = "https://firestore.googleapis.com/v1";
const FIREBASE_AUTH_BASE: &str = "https://identitytoolkit.googleapis.com/v1";

// ─── Shared helpers ───────────────────────────────────────────────────────────

/// Extract a string field from a Firestore document
fn get_str(doc: &Value, field: &str) -> String {
    doc["fields"][field]["stringValue"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}

// ─── Auth ─────────────────────────────────────────────────────────────────────

pub mod auth {
    use super::*;

    #[derive(Deserialize)]
    pub struct AuthResponse {
        pub id_token: String,
        pub local_id: String,
    }

    pub async fn sign_in(
        client: &reqwest::Client,
        api_key: &str,
        email: &str,
        password: &str,
    ) -> anyhow::Result<AuthResponse> {
        let url = format!(
            "{}/accounts:signInWithPassword?key={}",
            FIREBASE_AUTH_BASE, api_key
        );

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
}

// ─── Messages ─────────────────────────────────────────────────────────────────

pub mod messages {
    use super::*;

    /// Path: users/{uid}/chats/{chat_id}/messages/{msg_id}
    pub async fn save(
        client: &reqwest::Client,
        project_id: &str,
        id_token: &str,
        uid: &str,
        chat_id: &str,
        msg_id: &str,
        message: &Message,
        timestamp: &str,
        starred: bool,
        tokens: u64,
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
                    "timestamp": { "stringValue": timestamp },
                    "starred":   { "booleanValue": starred },
                    "tokens":    { "integerValue": tokens.to_string() }
                }
            }))
            .send()
            .await?;

        Ok(())
    }

    /// Returns last N messages sorted by timestamp
    pub async fn load_recent(
        client: &reqwest::Client,
        project_id: &str,
        id_token: &str,
        uid: &str,
        chat_id: &str,
        limit: usize,
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

        let mut pairs: Vec<(String, Message)> = resp["documents"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|doc| {
                let role = super::get_str(doc, "role");
                let content = super::get_str(doc, "content");
                let timestamp = super::get_str(doc, "timestamp");
                (timestamp, Message { role, content })
            })
            .collect();

        pairs.sort_by(|a, b| a.0.cmp(&b.0));

        let recent = pairs
            .into_iter()
            .rev()
            .take(limit)
            .rev()
            .map(|(_, m)| m)
            .collect();

        Ok(recent)
    }

    /// Returns all starred messages
    pub async fn load_starred(
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

        let mut starred: Vec<(String, Message)> = resp["documents"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter(|doc| {
                doc["fields"]["starred"]["booleanValue"]
                    .as_bool()
                    .unwrap_or(false)
            })
            .map(|doc| {
                let role = super::get_str(doc, "role");
                let content = super::get_str(doc, "content");
                let timestamp = super::get_str(doc, "timestamp");
                (timestamp, Message { role, content })
            })
            .collect();

        starred.sort_by(|a, b| a.0.cmp(&b.0));

        Ok(starred.into_iter().map(|(_, m)| m).collect())
    }

    /// Increment platform-wide token counter and message count
    pub async fn increment_platform_stats(
        client: &reqwest::Client,
        project_id: &str,
        id_token: &str,
        tokens: u64,
    ) -> anyhow::Result<()> {
        let url = format!(
            "{}/projects/{}/databases/(default)/documents/platform/stats",
            FIRESTORE_BASE, project_id
        );

        // First read current values
        let resp: Value = client
            .get(&url)
            .bearer_auth(id_token)
            .send()
            .await?
            .json()
            .await?;

        let current_tokens = resp["fields"]["tokens_used"]["integerValue"]
            .as_str()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        let current_messages = resp["fields"]["messages_count"]["integerValue"]
            .as_str()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        client
            .patch(&url)
            .bearer_auth(id_token)
            .json(&json!({
                "fields": {
                    "tokens_used":    { "integerValue": (current_tokens + tokens).to_string() },
                    "messages_count": { "integerValue": (current_messages + 1).to_string() }
                }
            }))
            .send()
            .await?;

        Ok(())
    }

// ─── Memory ───────────────────────────────────────────────────────────────────

pub mod memory {
    use super::*;

    /// Path: users/{uid}/memory/{point_id}
    pub async fn save(
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

    /// Returns all memory points sorted by timestamp ascending
    pub async fn load_all(
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

        let mut points: Vec<MemoryPoint> = resp["documents"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|doc| MemoryPoint {
                summary: super::get_str(doc, "summary"),
                timestamp: super::get_str(doc, "timestamp"),
            })
            .collect();

        points.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        Ok(points)
    }
}
