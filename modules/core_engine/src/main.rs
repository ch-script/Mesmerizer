mod db;

use std::fs::{OpenOptions};
use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::thread;
use rusqlite::Connection; // Added for database interaction

/// Handles an individual client connection in isolation
fn handle_client(stream: TcpStream) {
    let mut reader = BufReader::new(stream);
    let mut command_line = String::new();

    if let Ok(bytes_read) = reader.read_line(&mut command_line) {
        if bytes_read > 0 {
            let cmd = command_line.trim();

            if !cmd.is_empty() {
                println!("[sntd] Command received: {}", cmd);
                // in the following order:
                // Log the incoming job into the SQLite database
                let db_path = "/zroot/media/mesmerizer.db";

                if let Ok(conn) = Connection::open(db_path) {
                    match conn.execute(
                        "INSERT INTO downloads (command, status) VALUES (?1, ?2)",
                        (cmd, "running"),
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

                let log_file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&log_path);

                match log_file {
                    Ok(file) => {
                        let file_err = file.try_clone().expect("Failed to clone log file handle");
                        // Execute the command in the background
                        match Command::new("sh")
                            .arg("-c")
                            .arg(cmd)
                            .stdout(Stdio::from(file))
                            .stderr(Stdio::from(file_err))
                            .spawn()
                        {
                            Ok(child) => {
                                println!("[sntd] Process started in background with PID: {}. Logging to {}", child.id(), log_path);
                            }
                            Err(e) => {
                                eprintln!("[sntd] Failed to execute command: {}", e);
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("[sntd] Failed to open log file {}: {}. Command will not be executed.", log_path, e);
                    }
                }
            }
        }
    }
}

fn main() {
    let _db_conn = db::init_db().expect("❌ SQLITE FAILED HORRIBLY");
    let bind_addr = "0.0.0.0:9999";

    let listener = TcpListener::bind(bind_addr).unwrap_or_else(|e| {
        panic!("Fatal: Failed to bind to port 9999. Details: {}", e);
    });

    println!("---------------------------------------");
    println!("Mesmerizer Core Engine");
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
