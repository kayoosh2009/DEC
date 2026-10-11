use serde::{Deserialize, Serialize};
use std::fs;

// --- Structures ---

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MemoryPoint {
    pub summary: String,
    pub timestamp: String,
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    message: Message,
}

// --- Instruction loading ---

/// Always loads main.md, then auto-detects other .md files by keywords in user message
fn load_instructions(user_message: &str) -> String {
    let mut instructions = String::new();

    // Always load main.md
    if let Ok(main) = fs::read_to_string("Instruction/main.md") {
        instructions.push_str(&main);
        instructions.push_str("\n\n");
    }

    // Auto-load other .md files based on keywords in user message
    if let Ok(entries) = fs::read_dir("Instruction") {
        for entry in entries.flatten() {
            let path = entry.path();
            let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();

            if filename == "main.md" || !filename.ends_with(".md") {
                continue;
            }

            // Use filename stem as keyword (e.g. "code.md" -> "code")
            let keyword = filename.trim_end_matches(".md").to_lowercase();

            if user_message.to_lowercase().contains(&keyword) {
                if let Ok(content) = fs::read_to_string(&path) {
                    instructions.push_str(&content);
                    instructions.push_str("\n\n");
                }
            }
        }
    }

    instructions
}

// --- Main generation function ---

pub async fn generate_response(
    client: &reqwest::Client,
    ollama_url: &str,
    ollama_api_key: &str,
    model: &str,
    user_message: &str,
    recent_messages: Vec<Message>,   // last 5 messages from current chat
    memory_points: Vec<MemoryPoint>, // diary points from Firebase
) -> anyhow::Result<String> {
    let instructions = load_instructions(user_message);

    // Build system prompt: instructions + diary memory points
    let mut system_content = instructions;

    if !memory_points.is_empty() {
        system_content.push_str("## Chat diary (past memory points)\n");
        for point in &memory_points {
            system_content.push_str(&format!("- [{}] {}\n", point.timestamp, point.summary));
        }
    }

    // Assemble messages: system -> recent history -> current user message
    let mut messages: Vec<Message> = vec![Message {
        role: "system".to_string(),
        content: system_content,
    }];

    messages.extend(recent_messages);

    messages.push(Message {
        role: "user".to_string(),
        content: user_message.to_string(),
    });

    // Call Ollama
    let request = OllamaRequest {
        model: model.to_string(),
        messages,
        stream: false,
    };

    let response = client
        .post(format!("{}/api/chat", ollama_url))
        .header("Authorization", format!("Bearer {}", ollama_api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?
        .json::<OllamaResponse>()
        .await?;

    Ok(response.message.content)
}