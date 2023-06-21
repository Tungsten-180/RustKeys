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
enum Position {
    One,
    Two,
    Three,
    Four,
    Five,
}
struct KeyProcessor {
    short: [Option<OrderedKeyPress>; 2],
    long: [Option<OrderedKeyPress>; 5],
    position: Position,
}
impl KeyProcessor {
    fn new() -> Self {
        Self {
            short: [None, None],
            long: [None, None, None, None, None],
            position: Position::One,
        }
    }
    fn take(&mut self, event: OrderedKeyPress) {
        match event.value {
            Action::Held => return,
            Action::Down | Action::Up => self.keep(event),
        }
    }
    fn keep(&mut self, event: OrderedKeyPress) {
        println!("keeping:{:?}", event);
        match self.position {
            Position::One => {
                self.short[0] = Some(event);
                self.position = Position::Two;
                return;
            }
            Position::Two => {
                self.short[1] = Some(event);
                self.check();
                return;
            }
            Position::Three => {
                self.long[2] = Some(event);
                self.position = Position::Four;
                return;
            }
            Position::Four => {
                self.long[3] = Some(event);
                self.position = Position::Five;
                return;
            }
            Position::Five => {
                self.long[4] = Some(event);
                self.check();
                return;
            }
        }
    }
    fn check(&mut self) {
        let mut s = self.short.iter();
        let mut l = self.long.iter().peekable();
        match self.position {
            Position::Two => {
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
            Position::Five => {
                if l.clone().any(|i| i.is_none()) {
                    panic!();
                }
                let mut l = l.clone().map(|i| i.unwrap()).peekable();
                if l.clone().take(2).all(|item| item.key == Key::KEY_LEFTSHIFT) {
                    let a = l.clone().nth(3).unwrap().key;
                    let b = l.clone().nth(4).unwrap().key;
                    if a == l.clone().nth(5).unwrap().key && b != Key::KEY_LEFTSHIFT {
                        println!("shifted {:?}", b);
                    }
                } else {
                    for _ in 0..l.clone().count() {
                        if l.next().unwrap().key == l.peek().unwrap().key
                            && l.peek().unwrap().key != Key::KEY_LEFTSHIFT
                        {
                            println!("two found in five:{:?}", l.peek().unwrap().key);
                        }
                    }
                }
            }
            _ => panic!(),
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
