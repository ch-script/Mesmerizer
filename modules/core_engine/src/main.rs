mod db;

use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::thread;
use rusqlite::Connection;

/// Handles an individual client connection in isolation
fn handle_client(stream: TcpStream) {
    let mut reader = BufReader::new(stream);
    let mut command_line = String::new();

    if let Ok(bytes_read) = reader.read_line(&mut command_line) {
        if bytes_read > 0 {
            let cmd = command_line.trim().to_string();

            if !cmd.is_empty() {
                println!("[sntd] Command received: {}", cmd);

                let db_path = "/zroot/media/mesmerizer.db";

                if let Ok(conn) = Connection::open(db_path) {
                    match conn.execute(
                        "INSERT INTO downloads (command, status) VALUES (?1, ?2)",
                        (&cmd, "running"),
                    ) {
                        Ok(_) => println!("[SQLite] Successfully registered job in database."),
                        Err(e) => eprintln!("[SQLite] Failed to insert job: {}", e),
                    }
                } else {
                    eprintln!("[SQLite] Error connecting to database within thread.");
                }

                // Log data
                let log_dir = "/zroot/media/.data";
                let _ = std::fs::create_dir_all(log_dir);
                let log_path = format!("{}/daemon.log", log_dir);

                match Command::new("sh")
                    .arg("-c")
                    .arg(&cmd)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                {
                    Ok(mut child) => {
                        println!("[sntd] Process started in background with PID: {}", child.id());
                        println!("---------------------");

                        // Process Standard Output
                        let log_p1 = log_path.clone();
                        if let Some(mut stdout) = child.stdout.take() {
                            thread::spawn(move || {
                                if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_p1) {
                                    let mut buffer = [0; 1024];
                                    let mut line_start = true;
                                    loop {
                                        match stdout.read(&mut buffer) {
                                            Ok(0) => break, // EOF
                                            Ok(n) => {
                                                let chunk = String::from_utf8_lossy(&buffer[..n]);
                                                for c in chunk.chars() {
                                                    if line_start {
                                                        print!("log: ");
                                                        let _ = write!(file, "log: ");
                                                        line_start = false;
                                                    }
                                                    print!("{}", c);
                                                    let _ = write!(file, "{}", c);

                                                    if c == '\n' || c == '\r' {
                                                        line_start = true;
                                                        let _ = std::io::stdout().flush();
                                                    }
                                                }
                                            }
                                            Err(_) => break,
                                        }
                                    }
                                }
                            });
                        }

                        // Process Standard Error
                        let log_p2 = log_path.clone();
                        if let Some(mut stderr) = child.stderr.take() {
                            thread::spawn(move || {
                                if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_p2) {
                                    let mut buffer = [0; 1024];
                                    let mut line_start = true;
                                    loop {
                                        match stderr.read(&mut buffer) {
                                            Ok(0) => break,
                                            Ok(n) => {
                                                let chunk = String::from_utf8_lossy(&buffer[..n]);
                                                for c in chunk.chars() {
                                                    if line_start {
                                                        print!("log: [ERR] ");
                                                        let _ = write!(file, "log: [ERR] ");
                                                        line_start = false;
                                                    }
                                                    print!("{}", c);
                                                    let _ = write!(file, "{}", c);

                                                    if c == '\n' || c == '\r' {
                                                        line_start = true;
                                                        let _ = std::io::stdout().flush();
                                                    }
                                                }
                                            }
                                            Err(_) => break,
                                        }
                                    }
                                }
                            });
                        }
                    }
                    Err(e) => {
                        eprintln!("[sntd] Failed to execute command: {}", e);
                    }
                }
            }
        }
    }
}

fn main() {
    let _db_conn = db::init_db().expect("SQLITE FAILED HORRIBLY");
    let bind_addr = "0.0.0.0:9999";

    let listener = TcpListener::bind(bind_addr).unwrap_or_else(|e| {
        panic!("Fatal: Failed to bind to port 9999. Details: {}", e);
    });

    println!("---------------------------------------");
    println!("Mesmerizer Core Engine (sntd) Active");
    println!("Listening for connections on: {}", bind_addr);
    println!("---------------------------------------");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream);
                });
            }
            Err(e) => {
                eprintln!("[sntd] Error accepting incoming connection: {}", e);
            }
        }
    }
}
