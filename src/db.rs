use rusqlite::{params, Connection, Result};
use chrono::{DateTime, Local};

pub struct CommandLog {
    pub id: String,
    pub timestamp: DateTime<Local>,
    pub cwd: String,
    pub command: String,
    pub exit_code: i32,
    pub duration_secs: f64,
}

pub fn init_db(db_path: &str) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS command_logs (
            id TEXT PRIMARY KEY,
            timestamp TEXT NOT NULL,
            cwd TEXT NOT NULL,
            command TEXT NOT NULL,
            exit_code INTEGER NOT NULL,
            duration_secs REAL NOT NULL
        )",
        [],
    )?;
    Ok(conn)
}

pub fn insert_command_log(conn: &Connection, log: &CommandLog) -> Result<()> {
    conn.execute(
        "INSERT INTO command_logs (id, timestamp, cwd, command, exit_code, duration_secs)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            log.id,
            log.timestamp.to_rfc3339(),
            log.cwd,
            log.command,
            log.exit_code,
            log.duration_secs,
        ],
    )?;
    Ok(())
} 