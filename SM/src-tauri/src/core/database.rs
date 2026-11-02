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
        CREATE TABLE IF NOT EXISTS settings (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
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

/// Read a non-secret setting value. Secrets (API keys) never go here, see `ai_settings`.
pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>, AppError> {
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
    let mut rows = stmt.query(params![key])?;
    match rows.next()? {
        Some(row) => Ok(Some(row.get(0)?)),
        None => Ok(None),
    }
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_roundtrip() {
        let conn = open_memory_db().expect("in-memory db");
        assert_eq!(get_setting(&conn, "ai").unwrap(), None);
        set_setting(&conn, "ai", "{\"a\":1}").unwrap();
        set_setting(&conn, "ai", "{\"a\":2}").unwrap();
        assert_eq!(get_setting(&conn, "ai").unwrap().as_deref(), Some("{\"a\":2}"));
    }
}
