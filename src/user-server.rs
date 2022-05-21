use std::{
    env,
    io::{self, BufRead, BufReader, Write},
    os::unix::net::UnixListener,
    path::Path,
    process::Command,
};

mod config;

fn main() {
    let config_path = env::var("MACRO_KB_CONFIG").unwrap_or(format!(
        "{}/.config/macro-kb.conf",
        env::var("HOME").expect("Unable to determine home directory")
    ));
    let socket_path = env::var("MACRO_KB_SOCKET").unwrap_or("/tmp/macro-kb.sock".to_string());

    if Path::new(&socket_path).exists() {
        loop {
            println!("Socket already exists, delete it? [y/n]: ");
            let mut result = String::new();
            io::stdin()
                .read_line(&mut result)
                .expect("Failed to read line");
            if result.trim() == "y" {
                std::fs::remove_file(&socket_path).expect("Failed to delete socket");
                break;
            } else if result.trim() == "n" {
                println!("Failed to acquire socket");
                return;
            }
        }
    }

    let mut config = config::parse_config(&config_path).expect("Unable to parse config");
    let listener = UnixListener::bind(&socket_path).expect("Failed to bind to socket");

    println!("Waiting for root daemon to connect...");

    let mut stream = match listener.accept() {
        Ok((stream, _)) => stream,
        Err(why) => {
            println!("Failed to accept connection: {}", why);
            return;
        }
    };

    println!("Connected to root daemon");

    // Store a possible new config here to avoid borrow problems
    let mut new_config = None;

    for line in BufReader::new(stream.try_clone().expect("Failed to clone stream")).lines() {
        match new_config {
            Some(_new_config) => {
                config = _new_config;
                new_config = None;
            }
            None => (),
        }
        match line {
            Ok(message) => {
                // Identify messages between the server and the client
                println!("Got message `{}` from client", message);
                match config.get(&message) {
                    Some(commands) => {
                        for command in commands {
                            match command.as_str() {
                                "RELOAD" => {
                                    println!("Reloading config...");
                                    new_config = Some(
                                        config::parse_config(&config_path)
                                            .expect("Unable to parse config"),
                                    );
                                    stream
                                        .write_all(b"OK\n")
                                        .expect("Failed to write to socket");
                                    continue;
                                }
                                "EXIT" => {
                                    println!("Exiting");
                                    stream
                                        .write_all(b"EXIT\n")
                                        .expect("Failed to write to socket");
                                    // Delete the socket after we are done with it
                                    std::fs::remove_file(&socket_path)
                                        .expect("Failed to delete socket");
                                    return;
                                }
                                _ => (),
                            }
                            match Command::new("sh").arg("-c").arg(command).spawn() {
                                Ok(_) => (),
                                Err(why) => println!("Failed to execute command: {}", why),
                            }
                        }
                    }
                    None => (),
                }
                println!("Replying with OK");

                stream
                    .write_all(b"OK\n")
                    .expect("Failed to write to socket");
            }
            Err(why) => println!("Failed to read line: {}", why),
        }
    }
}
