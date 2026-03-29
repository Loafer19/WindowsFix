use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection, Result as SqlResult};

use crate::history::HistoryEntry;
use crate::models::ServiceInfo;

/// Path to the SQLite database file, placed next to the executable.
fn db_path() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().and_then(|p| p.parent()).map(|p| p.join("services.db")))
}

/// Open (or create) the application database and run migrations.
pub fn open_db() -> SqlResult<Connection> {
    let conn = match db_path() {
        Some(path) => Connection::open(&path)?,
        None => Connection::open_in_memory()?,
    };
    run_migrations(&conn)?;
    Ok(conn)
}

/// Create an in-memory database with migrations applied (used as a fallback).
pub fn open_memory_db() -> SqlResult<Connection> {
    let conn = Connection::open_in_memory()?;
    run_migrations(&conn)?;
    Ok(conn)
}

fn run_migrations(conn: &Connection) -> SqlResult<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS service_info (
            name        TEXT    PRIMARY KEY,
            description TEXT,
            explained   TEXT,
            recommendation TEXT,
            updated_at  INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS history (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            entry_json TEXT    NOT NULL,
            timestamp  INTEGER NOT NULL
        );
        ",
    )?;
    Ok(())
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Load all non-expired service info entries from the database.
pub fn load_service_info(conn: &Connection, ttl_secs: u64) -> HashMap<String, ServiceInfo> {
    let mut map = HashMap::new();
    let now = now_secs();

    let mut stmt = match conn.prepare(
        "SELECT name, description, explained, recommendation
         FROM service_info
         WHERE (? - updated_at) < ?",
    ) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to prepare service_info query: {}", e);
            return map;
        }
    };

    let rows = stmt.query_map(params![now, ttl_secs as i64], |row| {
        Ok((
            row.get::<_, String>(0)?,
            ServiceInfo {
                description: row.get(1)?,
                explained: row.get(2)?,
                recommendation: row.get(3)?,
            },
        ))
    });

    if let Ok(rows) = rows {
        for row in rows.flatten() {
            map.insert(row.0, row.1);
        }
    }

    map
}

/// Persist a single service info entry, updating its timestamp.
pub fn save_service_info(conn: &Connection, name: &str, info: &ServiceInfo) {
    if let Err(e) = conn.execute(
        "INSERT OR REPLACE INTO service_info
             (name, description, explained, recommendation, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![name, info.description, info.explained, info.recommendation, now_secs()],
    ) {
        eprintln!("Failed to save service info for '{}': {}", name, e);
    }
}

/// Persist all entries in the given map, updating each timestamp.
pub fn save_all_service_info(conn: &Connection, services_info: &HashMap<String, ServiceInfo>) {
    eprintln!("DEBUG: save_all_service_info called with {} entries", services_info.len());
    let tx = conn.unchecked_transaction().unwrap();
    let mut count = 0;
    for (name, info) in services_info {
        count += 1;
        if count % 50 == 0 {
            eprintln!("DEBUG: Saved {} service infos", count);
        }
        save_service_info(&tx, name, info);
    }
    tx.commit().unwrap();
    eprintln!("DEBUG: Finished saving all service infos");
}

/// Append a history entry to the database.
pub fn append_history(conn: &Connection, entry: &HistoryEntry) {
    let json = match serde_json::to_string(entry) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("Failed to serialize history entry: {}", e);
            return;
        }
    };
    if let Err(e) = conn.execute(
        "INSERT INTO history (entry_json, timestamp) VALUES (?1, ?2)",
        params![json, entry.timestamp() as i64],
    ) {
        eprintln!("Failed to save history entry: {}", e);
    }
}

/// Load the most recent `limit` history entries, newest first.
pub fn load_history(conn: &Connection, limit: u32) -> Vec<HistoryEntry> {
    let mut stmt = match conn.prepare(
        "SELECT entry_json FROM history ORDER BY timestamp DESC LIMIT ?",
    ) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to prepare history query: {}", e);
            return vec![];
        }
    };

    let json_vec: Vec<String> = match stmt.query_map(params![limit as i64], |row| row.get::<_, String>(0)) {
        Ok(rows) => rows.flatten().collect(),
        Err(_) => vec![],
    };
    json_vec.into_iter().filter_map(|json| serde_json::from_str(&json).ok()).collect()
}

/// Clear all history entries from the database.
pub fn clear_history(conn: &Connection) {
    if let Err(e) = conn.execute("DELETE FROM history", []) {
        eprintln!("Failed to clear history: {}", e);
    }
}
