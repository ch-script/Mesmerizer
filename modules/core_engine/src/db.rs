use rusqlite::{Connection, Result};

pub fn init_db() -> Result<Connection> {
    let conn = Connection::open("mesmerizer.db")?;
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

    println!("🗄️  [SQLite] Database 'mesmerizer.db' succesfuly initialized");
    Ok(conn)
}
