use rppal::gpio::{InputPin, Pin};
use log::warn;

#[derive(Copy, Clone, Debug)]
pub enum Pull {
    None,
    Up,
    Down
}

impl From<&'static str> for Pull {
    fn from(name: &'static str) -> Self {
        str_to_pull(name.to_ascii_lowercase().as_str())
    }
}

impl From<String> for Pull {
    fn from(name: String) -> Self {
        str_to_pull(name.to_ascii_lowercase().as_str())
    }
}

impl From<&String> for Pull {
    fn from(name: &String) -> Self {
        str_to_pull(name.to_ascii_lowercase().as_str())
    }
}

impl From<&Option<String>> for Pull {
    fn from(name: &Option<String>) -> Self {
        if let Some(name) = name {
            return str_to_pull(name.to_ascii_lowercase().as_str())
        }
        Pull::None
    }
}

#[inline(always)]
fn str_to_pull(name: &str) -> Pull {
    match name {
        "up" => Pull::Up,
        "down" => Pull::Down,
        "none" | _ => Pull::None
    }
}

pub fn make_input_pin(pin: Pin, pull: Pull) -> InputPin {
    match pull {
        Pull::Up => pin.into_input_pullup(),
        Pull::Down => pin.into_input_pulldown(),
        Pull::None => {
            warn!("Pin {} is floating. Consider using internal pull resistor instead.", pin.pin());
            pin.into_input()
        }
    }
}