use std::convert::TryInto;

use evdev::{Device, EventType, InputEvent, Key};
use keyboard::Kind;
mod keyboard;
impl std::convert::TryFrom<InputEvent> for OrderedKeyPress {
    type Error = ();
    fn try_from(value: InputEvent) -> Result<Self, Self::Error> {
        if value.event_type() != EventType::KEY {
            return Err(());
        }
        match value.value() {
            0 => {
                return Ok(OrderedKeyPress {
                    key: Key::new(value.code()),
                    value: Action::Up,
                })
            }
            1 => {
                return Ok(OrderedKeyPress {
                    key: Key::new(value.code()),
                    value: Action::Down,
                })
            }
            2 => {
                return Ok(OrderedKeyPress {
                    key: Key::new(value.code()),
                    value: Action::Held,
                })
            }
            _ => return Err(()),
        }
    }
}
struct Comparator<T> {
    last: Option<T>,
    value: bool,
}
impl<T: std::cmp::Eq + std::clone::Clone> Comparator<T> {
    fn new() -> Self {
        Self {
            value: true,
            last: None,
        }
    }
    fn cmp(&self, t: T) -> Self {
        match (self.value, self.last.clone()) {
            (false, None) => Self {
                last: None,
                value: false,
            },
            (true, None) => Self {
                last: Some(t),
                value: true,
            },
            (true, Some(prev)) if prev == t => Self {
                last: Some(t),
                value: true,
            },
            _ => Self {
                last: None,
                value: false,
            },
        }
    }
}
#[derive(Debug, Clone, Copy)]
enum Action {
    Down,
    Up,
    Held,
}
#[derive(Debug, Clone, Copy)]
struct OrderedKeyPress {
    key: Key,
    value: Action,
}
impl OrderedKeyPress {
    fn create(key: Key, value: Action) -> Self {
        Self { key, value }
    }
}
enum Commands {
    Empty,
}
#[derive(Debug, Clone, Copy)]
enum Lposition {
    Off,
    One,
    Two,
    Three,
    Four,
    Five,
}
#[derive(Debug, Clone, Copy)]
enum Sposition {
    Off,
    One,
    Two,
}
struct KeyProcessor {
    short: [Option<OrderedKeyPress>; 2],
    sposition: Sposition,
    long: [Option<OrderedKeyPress>; 5],
    lposition: Lposition,
}
impl KeyProcessor {
    fn new() -> Self {
        Self {
            short: [None, None],
            sposition: Sposition::One,
            long: [None, None, None, None, None],
            lposition: Lposition::Off,
        }
    }
    fn take(&mut self, event: OrderedKeyPress) {
        match event.value {
            Action::Held => return,
            Action::Down | Action::Up => self.keep(event),
        }
    }
    fn keep(&mut self, event: OrderedKeyPress) {
        match self.sposition {
            Sposition::One => {
                self.short[0] = Some(event);
                self.sposition = Sposition::Two;
                return;
            }
            Sposition::Two => {
                self.short[1] = Some(event);
                self.check();
                return;
            }
            Sposition::Off => {}
        }
        match self.lposition {
            Lposition::Off => {
                panic!()
            }
            Lposition::One => {
                self.short[0] = Some(event);
                self.lposition = Lposition::Off;
                self.sposition = Sposition::Two;
            }
            Lposition::Two => {
                self.short[1] = Some(event);
                self.lposition = Lposition::Off;
                self.sposition = Sposition::Two;
                self.check();
                return;
            }
            Lposition::Three => {
                self.long[2] = Some(event);
                self.lposition = Lposition::Four;
                return;
            }
            Lposition::Four => {
                self.long[3] = Some(event);
                self.lposition = Lposition::Five;
                return;
            }
            Lposition::Five => {
                self.long[4] = Some(event);
                self.check();
                return;
            }
        }
    }
    fn check(&mut self) {
        let mut s = self.short.iter();
        let l = self.long.iter();
        match (self.sposition, self.lposition) {
            (Sposition::Two, Lposition::Off) => {
                if s.by_ref().any(|item| item.is_none()) {
                    if s.as_ref()
                        .iter()
                        .fold(Comparator::<Key>::new(), |comp, new| {
                            comp.cmp(new.unwrap().key)
                        })
                        .value
                    {
                        println!("Key success:{:?}", s.next().unwrap().unwrap().key);
                    }
                }
            }
            (Sposition::Off, Lposition::Five) => {
                if l.take(2).all(|item| {
                    if let Some(a) = item {
                        a.key == Key::KEY_LEFTSHIFT
                    } else {
                        false
                    }
                }) {}
            }
            (..) => panic!(),
        }
    }
}
fn main() {
    let mut device = keyboard::get_keyboard(keyboard::CHERRYKEYBOARD);
    let mut sequence = KeyProcessor::new();
    loop {
        if let Ok(events) = device.fetch_events() {
            events
                .filter(|event| event.kind().is_key())
                .for_each(|keyevent| {
                    sequence.take(
                        keyevent
                            .try_into()
                            .ok()
                            .expect("input event failed try_into OrderedKeyPress"),
                    )
                })
        }
    }
}
