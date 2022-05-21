use std::{
    env,
    fs::File,
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
};

use evdev_rs::{enums::EventType, Device, GrabMode, ReadFlag};

mod config;

fn main() {
    let config_path = env::var("MACRO_KB_CONFIG").unwrap_or("/etc/macro-kb.conf".to_string());
    let socket_path = env::var("MACRO_KB_SOCKET").unwrap_or("/tmp/macro-kb.sock".to_string());
    let config = match config::parse_config(&config_path) {
        Ok(config) => config,
        Err(why) => {
            println!("Failed to parse config: {}", why);
            return;
        }
    };

    let mut device = None;
    let mut devices_nolisten = Vec::new();

    for (key, value) in config {
        if key == "DEVICE" && device.is_none() {
            if value.len() > 1 {
                println!("Too many devices specified, if you need to capture devices to avoid keypresses use the DEVICES_NOLISTEN key");
                return;
            }
            device = Some(
                Device::new_from_file(
                    File::open(&value[0])
                    .expect("Failed to open device file")
                )
                .expect("Failed to open device from file, is the path pointing to an actual input device?")
            );
        } else if key == "DEVICE_NOLISTEN" {
            for device_path in value {
                devices_nolisten.push(
                    Device::new_from_file(
                        File::open(device_path)
                        .expect("Failed to open device file")
                    )
                    .expect("Failed to open device from file, is the path pointing to an actual input device?")
                );
            }
        }
    }

    // Make sure the main device was captured
    let mut device = match device {
        Some(device) => device,
        None => {
            println!("No DEVICE key in config file, make sure it is specified");
            return;
        }
    };

    device
        .grab(GrabMode::Grab)
        .expect("Failed to capture device");

    for device in &mut devices_nolisten {
        device
            .grab(GrabMode::Grab)
            .expect("Failed to capture device");
    }

    let mut stream = loop {
        match UnixStream::connect(&socket_path) {
            Ok(stream) => break stream,
            Err(why) => {
                println!("Failed to connect to socket, retrying in 1 second: {}", why);
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    };

    println!("Connected to userspace daemon");

    let mut reader = BufReader::new(stream.try_clone().expect("Failed to clone stream")).lines();

    loop {
        match device.next_event(ReadFlag::NORMAL | ReadFlag::BLOCKING) {
            Ok(ev) => {
                if ev.1.is_type(&EventType::EV_KEY) && ev.1.value == 1 {
                    println!("Sending key: {}", ev.1.event_code);
                    stream
                        .write_all(format!("{}\n", ev.1.event_code).as_bytes())
                        .expect("Failed to write to socket");
                    // Get response from server
                    let message = reader.next().unwrap().expect("Failed to get response");
                    match message.as_str() {
                        "OK" => {
                            println!("Key sent successfully");
                        }
                        "EXIT" => {
                            println!("Received exit order, exiting...");
                            break;
                        }
                        _ => println!("Unknown response from server: {}", message),
                    }
                }
            }
            Err(why) => {
                println!("Error getting next event: {:?}", why);
            }
        }
    }
    device
        .grab(GrabMode::Ungrab)
        .expect("Failed to ungrab device");

    for device in &mut devices_nolisten {
        device
            .grab(GrabMode::Ungrab)
            .expect("Failed to ungrab device");
    }
}
