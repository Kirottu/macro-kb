use std::{fs::File, process::Command};
use evdev_rs::{Device, GrabMode, ReadFlag, enums::EventType};

fn main() {
    let file = File::open("/dev/input/by-id/usb-KB_USB_Keyboard-if01-event-kbd").unwrap();
    let file2 = File::open("/dev/input/by-id/usb-KB_USB_Keyboard-event-kbd").unwrap();

    let mut dev = Device::new_from_file(file).unwrap();
    let mut dev2 = Device::new_from_file(file2).unwrap();
    
    dev.grab(GrabMode::Grab).unwrap();
    dev2.grab(GrabMode::Grab).unwrap();
    
    let target_user = "kirottu";
    let target_tmux_session = "macro-kb-tmux";
    let target_bash_script = "/home/kirottu/.config/scripts/macro-kb.sh";

    loop {
        let ev = dev.next_event(ReadFlag::NORMAL | ReadFlag::BLOCKING).map(|val| val.1);
        match ev {
            Ok(ev) => {
                if ev.is_type(&EventType::EV_KEY) && ev.value == 1 {
                    let arg = format!("tmux send -t {} '{} {} &' ENTER", target_tmux_session, target_bash_script, ev.event_code);
                    Command::new("su")
                        .arg(target_user)
                        .arg("-c")
                        .arg(&arg)
                        .output()
                        .expect("Failed to run command");
                }
            }
            Err(why) => println!("Error getting next event: {:?}", why)
        }
    }
}
