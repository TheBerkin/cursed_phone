#![allow(dead_code)]

use crate::config::*;

#[cfg(feature = "rpi")]
use crate::gpio::*;

pub enum PhoneInputSignal {
    HookState(bool),
    Motion,
    Digit,
}

#[derive(Copy, Clone, Debug)]
pub enum PhoneType {
    Rotary,
    TouchTone,
    Unknown
}

impl PhoneType {
    fn from_name(name: &str) -> PhoneType {
        use PhoneType::*;
        match name {
            "rotary" => Rotary,
            "touchtone" => TouchTone,
            "unknown" | _ => Unknown
        }
    }
}

pub struct PhoneEngine {
    phone_type: PhoneType,
    on_hook: bool,
    dial_resting: bool,
    dial_pulse: bool,
    ring_state: bool,
    vibe_state: bool,
    pdd: f32,
    off_hook_delay: f32,
    #[cfg(feature = "rpi")]
    gpio: GpioInterface
}

impl PhoneEngine {
    /// Constructor for Phone on Raspberry Pi platforms.
    #[cfg(feature = "rpi")]
    pub fn new(config: &CursedConfig) -> Self {
        let phone_type = PhoneType::from_name(config.phone_type.as_str());
        let gpio = GpioInterface::new(phone_type, &config);

        Self {
            phone_type,
            on_hook: true,
            dial_resting: true,
            dial_pulse: false,
            ring_state: false,
            vibe_state: false,
            pdd: config.pdd,
            off_hook_delay: config.off_hook_delay,
            gpio
        }
    }

    /// Constructor for Phone on non-Pi platforms.
    #[cfg(not(feature = "rpi"))]
    pub fn new(config: &CursedConfig) -> Self {
        let phone_type = PhoneType::from_name(config.phone_type.as_str());

        Self {
            phone_type,
            on_hook: true,
            dial_resting: true,
            dial_pulse: false,
            ring_state: false,
            vibe_state: false,
            pdd: config.pdd,
            off_hook_delay: config.off_hook_delay
        }
    }

    pub fn dial_digit(&self, digit: char) {
        todo!()
    }

    pub fn set_on_hook(&self, hook_state: bool) {
        todo!()
    }

    fn on_pick_up(&self) {

    }

    fn on_hang_up(&self) {

    }
}

impl PhoneEngine {

}