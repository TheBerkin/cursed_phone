use rppal::gpio::{InputPin, Pin};

#[derive(Copy, Clone, Debug)]
pub enum Pull {
    None,
    Up,
    Down
}

impl From<&'static str> for Pull {
    fn from(name: &'static str) -> Self {
        match name.to_ascii_lowercase().as_str() {
            "up" => Pull::Up,
            "down" => Pull::Down,
            "none" | _ => Pull::None
        }
    }
}

impl From<String> for Pull {
    fn from(name: String) -> Self {
        match name.to_ascii_lowercase().as_str() {
            "up" => Pull::Up,
            "down" => Pull::Down,
            "none" | _ => Pull::None
        }
    }
}

impl From<&String> for Pull {
    fn from(name: &String) -> Self {
        match name.to_ascii_lowercase().as_str() {
            "up" => Pull::Up,
            "down" => Pull::Down,
            "none" | _ => Pull::None
        }
    }
}

impl From<&Option<String>> for Pull {
    fn from(name: &Option<String>) -> Self {
        if let Some(name) = name {
            return match name.to_ascii_lowercase().as_str() {
                "up" => Pull::Up,
                "down" => Pull::Down,
                "none" | _ => Pull::None
            }
        }
        Pull::None
    }
}

pub fn make_input_pin(pin: Pin, pull: Pull) -> InputPin {
    match pull {
        Pull::Up => pin.into_input_pullup(),
        Pull::Down => pin.into_input_pulldown(),
        Pull::None => pin.into_input()
    }
}