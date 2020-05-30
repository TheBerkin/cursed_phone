#![allow(dead_code)]
use crate::{CursedConfig, GpioPinsConfig};

#[cfg(feature = "rpi")]
struct GpioInterface {
    config: GpioPinsConfig
}

pub struct PhoneEngine {
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
        let mut gpio = GpioInterface {
            config: config.gpio_pins
        };

        Phone {
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
        PhoneEngine {
            on_hook: true,
            dial_resting: true,
            dial_pulse: false,
            ring_state: false,
            vibe_state: false,
            pdd: config.pdd,
            off_hook_delay: config.off_hook_delay
        }
    }

    pub fn inform_motion_detected() {
        todo!()
    }
}