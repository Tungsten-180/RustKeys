use evdev;
use evdev::InputEventKind;
use std::process::{Command, Stdio};
pub const CHERRYKEYBOARD: &str = "0003:046A:003B.0005";
pub trait Kind {
    fn is_key(&self) -> bool {
        false
    }
}
impl Kind for InputEventKind {
    fn is_key(&self) -> bool {
        match self {
            evdev::InputEventKind::Key(_) => return true,
            _ => return false,
        }
    }
}
pub fn get_keyboard(identifier: &str) -> evdev::Device {
    let get_devices_list = Command::new("cat")
        .arg("/proc/bus/input/devices")
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to get list of devices");
    let device_list = String::from_utf8(
        get_devices_list
            .wait_with_output()
            .expect("failed to get stdout from sudo cat")
            .stdout,
    )
    .expect("stdout returned err");
    let path = device_list
        .split("\n")
        .skip_while(|line| !line.contains(CHERRYKEYBOARD))
        .skip_while(|line| !line.contains("event"))
        .next()
        .expect("line is missing")
        .split_whitespace()
        .skip_while(|word| !word.contains("event"))
        .next()
        .expect("word event not found");
    if let Ok(mut device) = evdev::Device::open(format!("/dev/input/{}", path)) {
        device.grab().expect("grabing failed");
        device
    } else {
        panic!();
    }
}
