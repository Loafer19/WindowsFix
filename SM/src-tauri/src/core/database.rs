use std::path::PathBuf;

use rusqlite::{params, Connection};

use super::history::HistoryEntry;
use super::models::AppError;

fn db_path() -> Option<PathBuf> {
    std::env::current_exe().ok().and_then(|p| {
        p.parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("services.db"))
    })
}

pub fn open_db() -> Result<Connection, AppError> {
    let conn = match db_path() {
        Some(path) => Connection::open(&path)?,
        None => Connection::open_in_memory()?,
    };
    run_migrations(&conn)?;
    Ok(conn)
}

pub fn open_memory_db() -> Result<Connection, AppError> {
    let conn = Connection::open_in_memory()?;
    run_migrations(&conn)?;
    Ok(conn)
}

fn run_migrations(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS history (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            entry_json TEXT    NOT NULL,
            timestamp  INTEGER NOT NULL
        );
        ",
    )?;
    Ok(())
}

pub fn append_history(conn: &Connection, entry: &HistoryEntry) -> Result<(), AppError> {
    let json = serde_json::to_string(entry).map_err(|e| AppError::Io {
        message: format!("Failed to serialize history entry: {}", e),
    })?;
    conn.execute(
        "INSERT INTO history (entry_json, timestamp) VALUES (?1, ?2)",
        params![json, entry.timestamp() as i64],
    )
    .map_err(AppError::from)?;
    Ok(())
}

pub fn load_history(conn: &Connection, limit: u32) -> Vec<HistoryEntry> {
    let mut stmt =
        match conn.prepare("SELECT entry_json FROM history ORDER BY timestamp DESC LIMIT ?") {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to prepare history query: {}", e);
                return vec![];
            }
        };

    let json_vec: Vec<String> =
        match stmt.query_map(params![limit as i64], |row| row.get::<_, String>(0)) {
            Ok(rows) => rows.flatten().collect(),
            Err(_) => vec![],
        };
    json_vec
        .into_iter()
        .filter_map(|json| serde_json::from_str(&json).ok())
        .collect()
}

pub fn clear_history(conn: &Connection) -> Result<(), AppError> {
    conn.execute("DELETE FROM history", [])
        .map_err(AppError::from)?;
    Ok(())
}
