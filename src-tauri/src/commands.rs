use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::State;

use crate::adapters::AdapterRegistry;
use crate::db::queries::DbQueries;
use crate::indexer::Indexer;
use crate::models::*;

pub struct AppState {
    pub db: Arc<Mutex<rusqlite::Connection>>,
    pub registry: Arc<AdapterRegistry>,
    pub indexer: Arc<Indexer>,
}

#[tauri::command]
pub async fn get_sessions(
    state: State<'_, AppState>,
    filters: SessionFilters,
    offset: Option<u32>,
    limit: Option<u32>,
) -> Result<Vec<Session>, String> {
    let db = state.db.lock().await;
    let queries = DbQueries::new(&db);
    queries
        .get_sessions(&filters, offset.unwrap_or(0), limit.unwrap_or(100))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_session_messages(
    state: State<'_, AppState>,
    session_id: String,
    offset: Option<u32>,
    limit: Option<u32>,
) -> Result<Vec<Message>, String> {
    let db = state.db.lock().await;
    let queries = DbQueries::new(&db);
    queries
        .get_messages(&session_id, offset.unwrap_or(0), limit.unwrap_or(500))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_sessions(
    state: State<'_, AppState>,
    query: String,
    filters: SessionFilters,
    limit: Option<u32>,
) -> Result<Vec<Message>, String> {
    let db = state.db.lock().await;
    let queries = DbQueries::new(&db);
    queries
        .search_messages(&query, limit.unwrap_or(50))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_resume_command(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<String, String> {
    let db = state.db.lock().await;

    let session = {
        let mut stmt = db
            .prepare("SELECT id, agent, project_path FROM sessions WHERE id = ?1")
            .map_err(|e| e.to_string())?;
        stmt.query_row(rusqlite::params![session_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?.unwrap_or_default(),
            ))
        })
        .map_err(|e| e.to_string())
    }?;

    let adapter = state
        .registry
        .get(&session.1)
        .ok_or_else(|| format!("Adapter {} not found", session.1))?;

    Ok(adapter.resume_command(&session.0, &session.2))
}

#[tauri::command]
pub async fn launch_resume(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), String> {
    let cmd = get_resume_command(state, session_id).await?;

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("osascript")
            .arg("-e")
            .arg(format!(
                "tell application \"Terminal\" to do script \"{}\"",
                cmd.replace('"', "\\\"")
            ))
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("xterm -e {} &", cmd))
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "cmd", "/K", &cmd])
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_active_sessions(
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    state.indexer.check_active_sessions().await
}

#[tauri::command]
pub async fn reindex_all(
    state: State<'_, AppState>,
) -> Result<crate::indexer::IndexStats, String> {
    state.indexer.index_all().await
}
