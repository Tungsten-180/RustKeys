use evdev::InputEventKind;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
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
fn read_devices_file() -> Result<String, io::Error> {
    let path = Path::new("/proc/bus/input/devices");

    let mut file = File::open(&path).expect("file could not be opened");

    let mut s = String::new();

    file.read_to_string(&mut s)?;
    return Result::Ok(s);
}

pub fn get_devices_list() -> Vec<String> {
    let devices = read_devices_file()
        .expect("read_devices_failed")
        .split("\n\n")
        .map(|i| i.to_string())
        .collect();
    return devices;
}

pub fn get_keyboard<S: Into<String>>(event: S) -> Result<evdev::Device, std::io::Error> {
    let event: String = event.into();
    let deviceres = evdev::Device::open(format!("/dev/input/{}", event));
    return deviceres;
}
