

use crate::db::CommandLog;
use rusqlite::Connection;
use chrono::Local;

pub fn log_command(conn: &Connection, command: String, cwd: String, exit_code: i32, duration_secs: f64) {
    let log = CommandLog {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: Local::now(),
        cwd,
        command,
        exit_code,
        duration_secs,
    };
    crate::db::insert_command_log(conn, &log).expect("Failed to insert log");
}
