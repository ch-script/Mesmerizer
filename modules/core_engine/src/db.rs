use rusqlite::{Connection, Result};
use std::env;

pub fn init_db() -> Result<Connection> {
    // Fetch the HOME environment variable, fallback to current directory if it fails
    let home_dir = env::var("HOME").unwrap_or_else(|_| String::from("."));
    let db_path = format!("{}/mesmerizer.db", home_dir);

    let conn = Connection::open(&db_path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS downloads (
            id INTEGER PRIMARY KEY,
            command TEXT NOT NULL,
            status TEXT NOT NULL,
            started_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS watch_state (
            id INTEGER PRIMARY KEY,
            file_path TEXT UNIQUE NOT NULL,
            progress_seconds INTEGER NOT NULL,
            completed BOOLEAN NOT NULL DEFAULT 0
        )",
        [],
    )?;

    println!("🗄️  [SQLite] Database '{}' successfully initialized", db_path);
    Ok(conn)
}
