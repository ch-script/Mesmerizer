use rusqlite::{Connection, Result};

pub fn init_db() -> Result<Connection> {
    let db_path = "/zroot/media/mesmerizer.db";
    let conn = Connection::open(db_path)?;

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
