use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::path::{Path, PathBuf};

use super::{AgentAdapter, SessionLocation};
use crate::models::*;

pub struct CursorAdapter;

impl CursorAdapter {
    pub fn new() -> Self {
        Self
    }

    fn data_dir() -> Option<PathBuf> {
        let home = dirs::home_dir()?;
        let cursor_dir = home.join(".cursor");
        if cursor_dir.exists() {
            Some(cursor_dir)
        } else {
            None
        }
    }

    fn find_db_files(dir: &Path, locations: &mut Vec<SessionLocation>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if name.starts_with('.') {
                    continue;
                }
                if path.is_dir() {
                    Self::find_db_files(&path, locations);
                } else {
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    if ext == "db" || ext == "sqlite" {
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
}

#[async_trait]
impl AgentAdapter for CursorAdapter {
    fn id(&self) -> &str {
        "cursor"
    }

    fn name(&self) -> &str {
        "Cursor"
    }

    async fn detect(&self) -> bool {
        Self::data_dir().is_some()
    }

    async fn scan(&self) -> Vec<SessionLocation> {
        let Some(data_dir) = Self::data_dir() else {
            return Vec::new();
        };

        let mut locations = Vec::new();
        Self::find_db_files(&data_dir, &mut locations);
        locations
    }

    async fn parse_session(&self, path: &Path) -> Result<NormalizedSession, String> {
        let file_name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let session = Session {
            id: file_name.clone(),
            agent: AgentType::Cursor,
            title: format!("Cursor Session ({})", file_name),
            project_path: String::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            file_path: path.to_string_lossy().to_string(),
            is_active: false,
            message_count: 0,
        };

        Ok(NormalizedSession {
            session,
            messages: Vec::new(),
            attachments: Vec::new(),
        })
    }

    fn resume_command(&self, _session_id: &str, project_path: &str) -> String {
        format!("cursor {}", project_path)
    }

    async fn is_active(&self, _session_path: &Path) -> bool {
        false
    }
}
