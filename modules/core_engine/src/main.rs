mod db;

use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::process::Command;
use std::thread;
use rusqlite::Connection; // Added for database interaction
use std::env;

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
                let home_dir = env::var("HOME").unwrap_or_else(|_| String::from("."));
                let db_path = format!("{}/mesmerizer.db", home_dir);

                if let Ok(conn) = Connection::open(&db_path) {
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

                // Execute the command in the background
                match Command::new("sh")
                    .arg("-c")
                    .arg(cmd)
                    .spawn()
                {
                    Ok(child) => {
                        println!("[sntd] Process started in background with PID: {}", child.id());
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
