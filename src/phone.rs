#![allow(dead_code)]
use std::sync::{mpsc, Mutex, Arc};
use std::time::{Instant, Duration};
use crate::{CursedConfig, GpioPinsConfig};

#[cfg(feature = "rpi")]
use rppal::gpio::*;

trait Debounce<T> where T: Debounced {
    fn debounce(self, time: Duration) -> T;
}

trait Debounced {
    fn on_changed_async<C>(&mut self, callback: C) -> Result<()> 
        where C: FnMut(bool) + Send + 'static;
}

#[cfg(feature = "rpi")]
struct SoftInputPin {
    pin: InputPin,
    debounce_time: Arc<Mutex<Duration>>,
    last_changed: Arc<Mutex<Instant>>,
    last_state: Arc<Mutex<bool>>
}

impl SoftInputPin {
    fn new(pin: InputPin, debounce_time: Duration) -> Self {
        let last_changed = Arc::new(Mutex::new(Instant::now()));
        let debounce_time = Arc::new(Mutex::new(debounce_time));
        let last_state = Arc::new(Mutex::new(pin.is_high()));
        Self {
            pin,
            debounce_time,
            last_changed,
            last_state
        }
    }
}

#[cfg(feature = "rpi")]
impl Debounce<SoftInputPin> for InputPin {
    fn debounce(self, time: Duration) -> SoftInputPin {
        SoftInputPin::new(self, time)
    }
}

impl Debounced for SoftInputPin {
    fn on_changed_async<C>(&mut self, mut callback: C) -> Result<()> 
    where C: FnMut(bool) + Send + 'static {
        let last_changed = self.last_changed.clone();
        let debounce_time = self.debounce_time.clone();
        let last_state = self.last_state.clone();
        self.pin.set_async_interrupt(Trigger::Both, move |level| {
            let new_state = level == Level::High;
            let debounce_time = debounce_time.lock().unwrap();
            let mut last_changed = last_changed.lock().unwrap();
            let mut last_state = last_state.lock().unwrap();
            if last_changed.elapsed() > *debounce_time && new_state != *last_state {
                *last_changed = Instant::now();
                *last_state = new_state;
                callback(new_state);
            }
        })
    }
}

#[cfg(feature = "rpi")]
struct GpioInterface {
    gpio: Gpio,
    //dial_pulse_count: Arc<Mutex<u32>>,
    //dial_switch_state: Arc<Mutex<bool>>,
    in_hook: SoftInputPin,
    in_dial_switch: Option<InputPin>,
    in_dial_pulse: Option<InputPin>,
    in_motion: Option<InputPin>,
    in_keypad_rows: Option<[InputPin; 4]>,
    out_keypad_cols: Option<[InputPin; 3]>,
    out_ringer: Option<OutputPin>,
    out_vibe: Option<OutputPin>,
    config: GpioPinsConfig
}

#[cfg(feature = "rpi")]
impl GpioInterface {
    fn new(phone_type: PhoneType, config: &CursedConfig) -> GpioInterface {
        let gpio = Gpio::new().expect("Unable to initialize GPIO interface");
        let gpio_cfg = &config.gpio;

        // Register standard pins
        let in_hook = gpio.get(gpio_cfg.in_hook).unwrap().into_input_pulldown().debounce(Duration::from_millis(25));
        let in_motion = if config.enable_motion_sensor { Some(gpio.get(gpio_cfg.in_motion).unwrap().into_input_pulldown()) } else { None };
        let out_ringer = if config.enable_ringer { Some(gpio.get(gpio_cfg.out_ringer).unwrap().into_output()) } else { None };
        let out_vibe = if config.enable_vibration { Some(gpio.get(gpio_cfg.out_vibrate).unwrap().into_output()) } else { None };
        let in_keypad_rows = None;
        let out_keypad_cols = None;
        let mut in_dial_switch = None;
        let mut in_dial_pulse = None;

        // Register special GPIO pins for phone type
        match phone_type {
            PhoneType::Rotary => {
                in_dial_pulse = Some(gpio.get(gpio_cfg.in_dial_pulse).unwrap().into_input_pulldown());
                in_dial_switch = Some(gpio.get(gpio_cfg.in_dial_switch).unwrap().into_input_pulldown());
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
            in_keypad_rows,
            out_keypad_cols,
            out_ringer,
            out_vibe,
            config: *gpio_cfg
        }
    }
}

#[cfg(feature = "rpi")]
impl GpioInterface {
    fn listen(&mut self) -> Result<mpsc::Receiver<PhoneInputSignal>> {
        use PhoneInputSignal::*;
        let (tx, rx) = mpsc::channel();
        
        // On/Off-hook GPIO events
        let sender = tx.clone();
        self.in_hook.on_changed_async(move |state| {
            sender.send(HookState(state)).unwrap();
        })?;

        // Motion sensor
        let sender = tx.clone();
        if let Some(in_motion) = &mut self.in_motion {
            in_motion.set_async_interrupt(Trigger::RisingEdge, move |_| {
                sender.send(Motion).unwrap();
            })?;
        }

        Ok(rx)
    }
}

enum PhoneInputSignal {
    HookState(bool),
    Motion,
    Digit,
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

    fn on_pick_up(&self) {

    }

    fn on_hang_up(&self) {

    }
}

impl PhoneEngine {

}