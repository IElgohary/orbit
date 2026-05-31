use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::path::{Path, PathBuf};

use super::{AgentAdapter, SessionLocation};
use crate::models::*;

pub struct CodexAdapter;

impl CodexAdapter {
    pub fn new() -> Self {
        Self
    }

    fn data_dir() -> Option<PathBuf> {
        let home = dirs::home_dir()?;
        let codex_dir = home.join(".codex");
        if codex_dir.exists() {
            Some(codex_dir)
        } else {
            None
        }
    }

    fn scan_dir_recursive(dir: &Path, locations: &mut Vec<SessionLocation>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    if name.starts_with('.') || name == "node_modules" {
                        continue;
                    }
                    Self::scan_dir_recursive(&path, locations);
                } else if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                    let modified = std::fs::metadata(&path)
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .and_then(|t| {
                            DateTime::from_timestamp(
                                t.duration_since(std::time::UNIX_EPOCH)
                                    .ok()?
                                    .as_secs()
                                    as i64,
                                0,
                            )
                        })
                        .unwrap_or_default();
                    locations.push(SessionLocation {
                        path,
                        last_modified: modified,
                    });
                }
            }
        }
    }
}

#[async_trait]
impl AgentAdapter for CodexAdapter {
    fn id(&self) -> &str {
        "codex"
    }

    fn name(&self) -> &str {
        "Codex"
    }

    async fn detect(&self) -> bool {
        Self::data_dir().is_some()
    }

    async fn scan(&self) -> Vec<SessionLocation> {
        let Some(data_dir) = Self::data_dir() else {
            return Vec::new();
        };

        let mut locations = Vec::new();
        Self::scan_dir_recursive(&data_dir, &mut locations);
        locations
    }

    async fn parse_session(&self, path: &Path) -> Result<NormalizedSession, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read: {}", e))?;

        let file_name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let mut messages = Vec::new();
        let mut title = String::from("Untitled");
        let mut seq: u32 = 0;
        let mut created_at = Utc::now();
        let mut updated_at = Utc::now();

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let json: serde_json::Value = match serde_json::from_str(line) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let role_str = json.get("role").and_then(|r| r.as_str()).unwrap_or("");
            let msg_content = json
                .get("content")
                .and_then(|c| {
                    if c.is_string() {
                        c.as_str().map(|s| s.to_string())
                    } else {
                        Some(c.to_string())
                    }
                })
                .unwrap_or_default();

            let role = match role_str {
                "user" | "human" => MessageRole::User,
                "assistant" => MessageRole::Assistant,
                "system" => MessageRole::System,
                "tool" => MessageRole::Tool,
                _ => continue,
            };

            if seq == 0 && role == MessageRole::User && !msg_content.is_empty() {
                title = msg_content.chars().take(100).collect();
            }

            let timestamp = json
                .get("timestamp")
                .and_then(|t| t.as_str())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc));

            if seq == 0 {
                created_at = timestamp.unwrap_or(Utc::now());
            }
            updated_at = timestamp.unwrap_or(Utc::now());

            messages.push(Message {
                id: uuid::Uuid::new_v4().to_string(),
                session_id: file_name.clone(),
                role,
                content: msg_content,
                timestamp,
                sequence: seq,
                tool_name: json
                    .get("tool_name")
                    .and_then(|t| t.as_str())
                    .map(|s| s.to_string()),
                tool_input: json.get("tool_input").map(|i| i.to_string()),
                tool_output: json.get("tool_output").map(|o| o.to_string()),
            });
            seq += 1;
        }

        let session = Session {
            id: file_name,
            agent: AgentType::Codex,
            title,
            project_path: String::new(),
            created_at,
            updated_at,
            file_path: path.to_string_lossy().to_string(),
            is_active: false,
            message_count: messages.len() as u32,
        };

        Ok(NormalizedSession {
            session,
            messages,
            attachments: Vec::new(),
        })
    }

    fn resume_command(&self, session_id: &str, _project_path: &str) -> String {
        format!("codex --resume {}", session_id)
    }

    async fn is_active(&self, _session_path: &Path) -> bool {
        false
    }
}
