use crate::models::*;

#[tauri::command]
pub fn get_sessions(_filters: Option<SessionFilters>, _offset: Option<u32>, _limit: Option<u32>) -> Result<Vec<Session>, String> {
    Ok(Vec::new())
}

#[tauri::command]
pub fn get_session_messages(_session_id: String, _offset: Option<u32>, _limit: Option<u32>) -> Result<Vec<Message>, String> {
    Ok(Vec::new())
}

#[tauri::command]
pub fn search_sessions(_query: String, _limit: Option<u32>) -> Result<Vec<Session>, String> {
    Ok(Vec::new())
}

#[tauri::command]
pub fn get_resume_command(_session_id: String) -> Result<String, String> {
    Ok(String::new())
}

#[tauri::command]
pub fn launch_resume(_session_id: String) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub fn get_active_sessions() -> Result<Vec<Session>, String> {
    Ok(Vec::new())
}

#[tauri::command]
pub fn reindex_all() -> Result<(), String> {
    Ok(())
}
