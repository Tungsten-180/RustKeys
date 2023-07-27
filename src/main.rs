use core::fmt;
use evdev::{Device, EventType, InputEvent, Key};
use std::{
    convert::TryInto,
    io::{self, Write},
};
mod keyboard;
use keyboard::{get_devices_list, get_keyboard, Kind};
trait NewTrait {
    fn select(&self) -> String;
}

impl NewTrait for Vec<String> {
    fn select(&self) -> String {
        let s = self.clone();
        let mut index: Option<usize> = None;
        let mut user_answer = String::new();
        while index.is_none() {
            for x in 0..s.len() {
                println!("{}. -------\n{}", x, s.get(x).unwrap());
            }

            print!("select one: ");
            Write::flush(&mut io::stdout()).unwrap();
            let input = std::io::stdin().read_line(&mut user_answer).unwrap();

            match user_answer.trim().parse::<usize>() {
                Err(err) => {
                    eprintln!("{}:{}", err, user_answer);
                    user_answer.clear();
                }
                Ok(number) if s.get(number).is_some() => index = Some(number),
                Ok(number) => {
                    eprintln!("{} is not a valid option", number);
                    user_answer.clear();
                }
            }
        }
        s.get(index.unwrap()).unwrap().clone()
    }
}

fn main() {
    let mut event: Option<String> = None;
    while &event.is_some() == &false {
        let mut device_list = get_devices_list();
        device_list = device_list
            .into_iter()
            .filter(|devicestr| {
                devicestr.contains("kbd")
                    && devicestr.contains("usb")
                    && devicestr.contains("input0")
            })
            .collect();
        event = device_list
            .select()
            .split_whitespace()
            .map(|i| i.to_string())
            .skip_while(|word| !word.contains("event"))
            .next();
    }
    match get_keyboard(event.unwrap()) {
        Err(_) => panic!("could not grab keyboard"),
        Ok(mut device) => {
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
    }
}

struct NotKeyPress(InputEvent);

impl fmt::Display for NotKeyPress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "InputEvent was not a key press")
    }
}

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

#[derive(Debug)]
struct Comparator<T> {
    last: Option<T>,
    value: bool,
}
impl<T: std::cmp::Eq + std::clone::Clone + std::fmt::Debug> Comparator<T> {
    fn new() -> Self {
        Self {
            value: true,
            last: None,
        }
    }
    fn cmp(&self, t: T) -> Self {
        println!("comp:{:?}", self);
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
    Six,
}

struct KeyProcessor {
    long: [Option<OrderedKeyPress>; 6],
    position: Position,
}
impl KeyProcessor {
    fn new() -> Self {
        Self {
            long: [None, None, None, None, None, None],
            position: Position::One,
        }
    }
    fn clear(&mut self) {
        self.long = [None, None, None, None, None, None];
        self.position = Position::One;
    }
    fn take(&mut self, event: OrderedKeyPress) {
        match event.value {
            Action::Held => return,
            Action::Down | Action::Up => self.keep(event),
        }
    }
    fn keep(&mut self, event: OrderedKeyPress) {
        match self.position {
            Position::One => {
                self.long[0] = Some(event);
                self.position = Position::Two;
                return;
            }
            Position::Two => {
                self.long[1] = Some(event);
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
                self.position = Position::Six;
                return;
            }
            Position::Six => {
                self.long[5] = Some(event);
                self.check();
                return;
            }
        }
    }
    fn check(&mut self) {
        let l = self.long.iter();
        match self.position {
            Position::Two => {
                let short = [self.long[0], self.long[1]];
                if short.iter().all(|item| item.is_some()) {
                    println!("{:?}", short);
                    if short
                        .iter()
                        .fold(Comparator::<Key>::new(), |comp, new| {
                            comp.cmp(new.unwrap().key)
                        })
                        .value
                        == true
                        && short[0].unwrap().key != Key::KEY_LEFTCTRL
                    {
                        println!("Key success:{:?}", short.iter().next());
                        self.clear();
                    } else {
                        self.position = Position::Three;
                    }
                } else {
                    panic!()
                }
            }
            Position::Six => {
                println!("five:{:?}", l.clone());
                if l.clone().any(|i| i.is_none()) {
                    panic!();
                }
                let l = l.clone().map(|i| i.unwrap()).peekable();
                if l.clone().take(2).all(|item| item.key == Key::KEY_LEFTCTRL) {
                    let a = self.long[2].unwrap().key;
                    let b = self.long[3].unwrap().key;
                    if a == self.long[4].unwrap().key && b != Key::KEY_LEFTCTRL {
                        println!("shifted {:?}", b);
                        self.clear()
                    }
                } else {
                    self.clear()
                }
            }
            _ => panic!(),
        }
    }
}
