use rusqlite::{params, Connection, Result};
use crate::models::*;

pub struct DbQueries<'a> {
    conn: &'a Connection,
}

impl<'a> DbQueries<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn get_source_hash(&self, file_path: &str) -> Result<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT source_hash FROM sessions WHERE file_path = ?1")?;
        let result = stmt
            .query_row(params![file_path], |row| row.get::<_, String>(0))
            .ok();
        Ok(result)
    }

    pub fn upsert_session(&self, session: &Session) -> Result<()> {
        self.conn.execute(
            "INSERT INTO sessions (id, agent, title, project_path, created_at, updated_at, file_path, is_active, message_count, source_hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                project_path = excluded.project_path,
                updated_at = excluded.updated_at,
                is_active = excluded.is_active,
                message_count = excluded.message_count,
                source_hash = excluded.source_hash",
            params![
                session.id,
                session.agent.as_str(),
                session.title,
                session.project_path,
                session.created_at.to_rfc3339(),
                session.updated_at.to_rfc3339(),
                session.file_path,
                session.is_active as i32,
                session.message_count,
                String::new(),
            ],
        )?;
        Ok(())
    }

    pub fn set_source_hash(&self, session_id: &str, hash: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE sessions SET source_hash = ?1 WHERE id = ?2",
            params![hash, session_id],
        )?;
        Ok(())
    }

    pub fn delete_session_messages(&self, session_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM messages WHERE session_id = ?1",
            params![session_id],
        )?;
        Ok(())
    }

    pub fn insert_message(&self, msg: &Message) -> Result<()> {
        self.conn.execute(
            "INSERT INTO messages (id, session_id, role, content, timestamp, sequence, tool_name, tool_input, tool_output)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                msg.id,
                msg.session_id,
                msg.role.as_str(),
                msg.content,
                msg.timestamp.map(|t| t.to_rfc3339()),
                msg.sequence,
                msg.tool_name,
                msg.tool_input,
                msg.tool_output,
            ],
        )?;
        Ok(())
    }

    pub fn get_sessions(
        &self,
        filters: &SessionFilters,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<Session>> {
        let mut sql = String::from(
            "SELECT id, agent, title, project_path, created_at, updated_at, file_path, is_active, message_count FROM sessions WHERE 1=1",
        );
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let mut param_idx = 1;

        if let Some(ref agent) = filters.agent {
            sql.push_str(&format!(" AND agent = ?{}", param_idx));
            param_values.push(Box::new(agent.clone()));
            param_idx += 1;
        }

        if let Some(ref project) = filters.project_path {
            sql.push_str(&format!(" AND project_path = ?{}", param_idx));
            param_values.push(Box::new(project.clone()));
            param_idx += 1;
        }

        if let Some(active) = filters.is_active {
            sql.push_str(&format!(" AND is_active = ?{}", param_idx));
            param_values.push(Box::new(active as i32));
            param_idx += 1;
        }

        if let Some(ref query) = filters.query {
            sql.push_str(&format!(
                " AND id IN (SELECT session_id FROM messages_fts WHERE messages_fts MATCH ?{} UNION SELECT s.id FROM sessions s WHERE s.title LIKE '%' || ?{} || '%')",
                param_idx, param_idx
            ));
            param_values.push(Box::new(query.clone()));
            param_idx += 1;
        }

        sql.push_str(" ORDER BY updated_at DESC");
        sql.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

        let mut stmt = self.conn.prepare(&sql)?;
        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();

        let sessions = stmt
            .query_map(params_refs.as_slice(), |row| {
                Ok(Session {
                    id: row.get(0)?,
                    agent: AgentType::from_str(&row.get::<_, String>(1)?).unwrap_or(AgentType::Claude),
                    title: row.get(2)?,
                    project_path: row.get(3)?,
                    created_at: row.get::<_, String>(4).ok().and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|dt| dt.with_timezone(&chrono::Utc)).unwrap_or_default(),
                    updated_at: row.get::<_, String>(5).ok().and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|dt| dt.with_timezone(&chrono::Utc)).unwrap_or_default(),
                    file_path: row.get(6)?,
                    is_active: row.get::<_, i32>(7)? != 0,
                    message_count: row.get(8)?,
                })
            })?
            .filter_map(|s| s.ok())
            .collect();

        Ok(sessions)
    }

    pub fn get_messages(
        &self,
        session_id: &str,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<Message>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, role, content, timestamp, sequence, tool_name, tool_input, tool_output
             FROM messages WHERE session_id = ?1 ORDER BY sequence ASC LIMIT ?2 OFFSET ?3",
        )?;

        let messages = stmt
            .query_map(params![session_id, limit, offset], |row| {
                let ts_str: Option<String> = row.get(4)?;
                let timestamp = ts_str
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc));
                Ok(Message {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    role: MessageRole::from_str(&row.get::<_, String>(2)?).unwrap_or(MessageRole::User),
                    content: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                    timestamp,
                    sequence: row.get(5)?,
                    tool_name: row.get(6)?,
                    tool_input: row.get(7)?,
                    tool_output: row.get(8)?,
                })
            })?
            .filter_map(|m| m.ok())
            .collect();

        Ok(messages)
    }

    pub fn mark_stale_sessions(&self, active_paths: &[String]) -> Result<u64> {
        let placeholders: Vec<String> = active_paths.iter().enumerate().map(|(i, _)| format!("?{}", i + 1)).collect();
        let sql = if active_paths.is_empty() {
            "DELETE FROM sessions WHERE 1=1".to_string()
        } else {
            format!(
                "DELETE FROM sessions WHERE file_path NOT IN ({})",
                placeholders.join(", ")
            )
        };

        let params: Vec<Box<dyn rusqlite::types::ToSql>> = active_paths
            .iter()
            .map(|p| Box::new(p.clone()) as Box<dyn rusqlite::types::ToSql>)
            .collect();
        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            params.iter().map(|p| p.as_ref()).collect();

        self.conn.execute(&sql, params_refs.as_slice()).map(|n| n as u64)
    }

    pub fn search_messages(&self, query: &str, limit: u32) -> Result<Vec<Message>> {
        let mut stmt = self.conn.prepare(
            "SELECT m.id, m.session_id, m.role, m.content, m.timestamp, m.sequence, m.tool_name, m.tool_input, m.tool_output
             FROM messages m
             INNER JOIN messages_fts fts ON m.rowid = fts.rowid
             WHERE messages_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2",
        )?;

        let messages = stmt
            .query_map(params![query, limit], |row| {
                let ts_str: Option<String> = row.get(4)?;
                let timestamp = ts_str
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc));
                Ok(Message {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    role: MessageRole::from_str(&row.get::<_, String>(2)?).unwrap_or(MessageRole::User),
                    content: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                    timestamp,
                    sequence: row.get(5)?,
                    tool_name: row.get(6)?,
                    tool_input: row.get(7)?,
                    tool_output: row.get(8)?,
                })
            })?
            .filter_map(|m| m.ok())
            .collect();

        Ok(messages)
    }

    pub fn get_active_session_ids(&self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM sessions WHERE is_active = 1")?;
        let ids = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(|id| id.ok())
            .collect();
        Ok(ids)
    }

    pub fn set_session_active(&self, session_id: &str, active: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE sessions SET is_active = ?1 WHERE id = ?2",
            params![active as i32, session_id],
        )?;
        Ok(())
    }

    pub fn rebuild_fts(&self) -> Result<()> {
        self.conn.execute_batch(
            "INSERT INTO messages_fts(messages_fts) VALUES('rebuild');"
        )?;
        Ok(())
    }
}
