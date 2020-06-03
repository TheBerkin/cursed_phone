#![allow(dead_code)]
use crate::{CursedConfig, GpioPinsConfig};

#[cfg(feature = "rpi")]
use rppal::gpio::{Gpio, InputPin, OutputPin};

#[cfg(feature = "rpi")]
struct GpioInterface {
    gpio: Gpio,
    in_hook: InputPin,
    in_dial_switch: Option<InputPin>,
    in_dial_pulse: Option<InputPin>,
    in_motion: Option<InputPin>,
    out_ringer: Option<OutputPin>,
    out_vibe: Option<OutputPin>,
    config: GpioPinsConfig
}

#[cfg(feature = "rpi")]
impl GpioInterface {
    fn new(phone_type: PhoneType, config: &CursedConfig) -> GpioInterface {
        let gpio = Gpio::new().expect("Unable to initialize GPIO interface");

        // Register standard pins
        let in_hook = gpio.get(config.gpio_pins.in_hook).unwrap().into_input_pulldown();
        let in_motion = if config.enable_motion_sensor { Some(gpio.get(config.gpio_pins.in_motion).unwrap().into_input_pulldown()) } else { None };
        let out_ringer = if config.enable_ringer { Some(gpio.get(config.gpio_pins.out_ringer).unwrap().into_output()) } else { None };
        let out_vibe = if config.enable_vibration { Some(gpio.get(config.gpio_pins.out_vibrate).unwrap().into_output()) } else { None };
        let mut in_dial_switch = None;
        let mut in_dial_pulse = None;

        // Register special GPIO pins for phone type
        match phone_type {
            PhoneType::Rotary => {
                in_dial_pulse = Some(gpio.get(config.gpio_pins.in_dial_pulse).unwrap().into_input_pulldown());
                in_dial_switch = Some(gpio.get(config.gpio_pins.in_dial_switch).unwrap().into_input_pulldown());
            },
            PhoneType::TouchTone => {
                todo!()
            },
            PhoneType::Unknown => {}
        }

        GpioInterface {
            gpio,
            in_hook,
            in_dial_switch,
            in_dial_pulse,
            in_motion,
            out_ringer,
            out_vibe,
            config: config.gpio_pins
        }
    }

    fn bind(&self, phone: &mut PhoneEngine) {
        
    }
}

#[cfg(feature = "rpi")]
impl GpioInterface {

}

#[derive(Copy, Clone, Debug)]
enum PhoneType {
    Rotary,
    TouchTone,
    Unknown
}

impl PhoneType {
    fn from_name(name: &str) -> PhoneType {
        match name {
            "rotary" => PhoneType::Rotary,
            "touchtone" => PhoneType::TouchTone,
            "unknown"|_ => PhoneType::Unknown
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
        let mut gpio = GpioInterface::new(phone_type, &config);

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
}

impl PhoneEngine {

}