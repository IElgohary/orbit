use rusqlite::Connection;

pub fn init_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            agent TEXT NOT NULL,
            title TEXT,
            project_path TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            file_path TEXT NOT NULL,
            is_active INTEGER NOT NULL DEFAULT 0,
            message_count INTEGER NOT NULL DEFAULT 0,
            source_hash TEXT
        );

        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
            role TEXT NOT NULL,
            content TEXT,
            timestamp TEXT,
            sequence INTEGER NOT NULL,
            tool_name TEXT,
            tool_input TEXT,
            tool_output TEXT
        );

        CREATE TABLE IF NOT EXISTS attachments (
            id TEXT PRIMARY KEY,
            message_id TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
            attachment_type TEXT NOT NULL,
            path TEXT NOT NULL,
            mime_type TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_sessions_agent ON sessions(agent);
        CREATE INDEX IF NOT EXISTS idx_sessions_project ON sessions(project_path);
        CREATE INDEX IF NOT EXISTS idx_sessions_updated ON sessions(updated_at DESC);
        CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id, sequence);
        ",
    )?;

    conn.execute_batch(
        "
        CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
            content,
            tool_input,
            tool_output,
            content=messages,
            content_rowid=rowid
        );
        ",
    )?;

    Ok(())
}
