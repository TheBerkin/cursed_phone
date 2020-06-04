#![cfg(feature = "rpi")]
#![allow(dead_code)]

use std::sync::{mpsc, Mutex, Arc};
use std::time::{Instant, Duration};
use rppal::gpio::*;
use crate::config::*;
use crate::phone::*;

/// Enables a digital input to be wrapped into a debounced input.
trait Debounce<T> where T: Debounced {
    fn debounce(self, time: Duration) -> T;
}

/// Represents a debounced digital input.
trait Debounced {
    fn on_changed<C>(&mut self, callback: C) -> Result<()> 
        where C: FnMut(bool) + Send + 'static;

    fn is_high(&self) -> bool;

    fn is_low(&self) -> bool;

    fn set_bounce_time(&mut self, time: Duration);
}

/// Simple wrapper around `rppal::gpio::pin::InputPin` to add debouncing.
struct SoftInputPin {
    pin: InputPin,
    bounce_time: Arc<Mutex<Duration>>,
    last_changed: Arc<Mutex<Instant>>,
    last_state: Arc<Mutex<bool>>
}

impl SoftInputPin {
    fn new(mut pin: InputPin, bounce_time: Duration) -> Self {
        let last_changed = Arc::new(Mutex::new(Instant::now()));
        let bounce_time = Arc::new(Mutex::new(bounce_time));
        let last_state = Arc::new(Mutex::new(pin.is_high()));
        pin.set_interrupt(Trigger::Both).unwrap();
        Self {
            pin,
            bounce_time,
            last_changed,
            last_state
        }
    }
}

impl Debounce<SoftInputPin> for InputPin {
    fn debounce(self, time: Duration) -> SoftInputPin {
        SoftInputPin::new(self, time)
    }
}

impl Debounced for SoftInputPin {
    fn on_changed<C>(&mut self, mut callback: C) -> Result<()> 
    where C: FnMut(bool) + Send + 'static {
        let last_changed = self.last_changed.clone();
        let bounce_time = self.bounce_time.clone();
        let last_state = self.last_state.clone();
        self.pin.set_async_interrupt(Trigger::Both, move |level| {
            let new_state = level == Level::High;
            let bounce_time = bounce_time.lock().unwrap();
            let mut last_changed = last_changed.lock().unwrap();
            let mut last_state = last_state.lock().unwrap();
            if last_changed.elapsed() > *bounce_time && new_state != *last_state {
                *last_changed = Instant::now();
                *last_state = new_state;
                callback(new_state);
            }
        })
    }

    fn is_high(&self) -> bool {
        let mut last_changed = self.last_changed.lock().unwrap();
        let mut last_state = self.last_state.lock().unwrap();
        let bounce_time = self.bounce_time.lock().unwrap();
        if last_changed.elapsed() < *bounce_time {
            return *last_state;
        }
        let new_state = self.pin.is_high();
        *last_state = new_state;
        *last_changed = Instant::now();
        new_state
    }

    #[inline]
    fn is_low(&self) -> bool {
        !self.is_high()
    }

    fn set_bounce_time(&mut self, time: Duration) {
        let mut bounce_time = self.bounce_time.lock().unwrap();
        *bounce_time = time;
    }
}

/// Provides an interface for phone-related GPIO pins.
pub struct GpioInterface {
    gpio: Gpio,
    /// Number of pulses since the dial switch last opened.
    dial_pulse_count: Arc<Mutex<u32>>,
    /// Position of the dial switch (true when at resting).
    dial_switch_state: Arc<Mutex<bool>>,
    /// Pin for switch hook input.
    in_hook: SoftInputPin,
    /// Pin for dial switch input.
    in_dial_switch: Option<SoftInputPin>,
    /// Pin for dial pulse switch input.
    in_dial_pulse: Option<SoftInputPin>,
    /// Pin for motion detector input.
    in_motion: Option<SoftInputPin>,
    /// Pins for keypad row inputs.
    in_keypad_rows: Option<[SoftInputPin; 4]>,
    /// Pins for keypad column outputs.
    out_keypad_cols: Option<[OutputPin; 3]>,
    /// Pin for ringer output.
    out_ringer: Option<OutputPin>,
    /// Pin for vibration motor output.
    out_vibe: Option<OutputPin>,
    /// Copy of GPIO pin config used to initialize pins.
    config: GpioConfig
}

fn gen_optional_soft_input(gpio: &Gpio, enable: Option<bool>, pin: Option<u8>, debounce: Option<u64>) -> Option<SoftInputPin> {
    if enable.unwrap_or(false) {
        if let Some(pin) = pin {
            let input = gpio.get(pin).unwrap()
                .into_input_pulldown()
                .debounce(Duration::from_millis(debounce.unwrap_or(0)));
            return Some(input);
        }
    }
    None
}

fn gen_required_soft_input(gpio: &Gpio, pin: u8, debounce: Option<u64>) -> SoftInputPin {
    gpio.get(pin).unwrap().into_input_pulldown().debounce(Duration::from_millis(debounce.unwrap_or(0)))
}

fn gen_optional_output(gpio: &Gpio, enable: Option<bool>, pin: Option<u8>) -> Option<OutputPin> {
    if enable.unwrap_or(false) {
        if let Some(pin) = pin {
            return Some(gpio.get(pin).unwrap().into_output());
        }
    }
    None
}

fn gen_required_output(gpio: &Gpio, pin: u8) -> OutputPin {
    gpio.get(pin).unwrap().into_output()
}

impl GpioInterface {
    pub fn new(phone_type: PhoneType, config: &CursedConfig) -> GpioInterface {
        use PhoneType::*;
        let gpio = Gpio::new().expect("Unable to initialize GPIO interface");
        let inputs = &config.gpio.inputs;
        let outputs = &config.gpio.outputs;

        // Register standard GPIO pins
        let in_hook = gpio.get(inputs.pin_hook).unwrap()
            .into_input_pulldown()
            .debounce(Duration::from_millis(inputs.pin_hook_bounce_ms.unwrap_or(0)));
        let in_motion = gen_optional_soft_input(&gpio, config.enable_motion_sensor, inputs.pin_motion, inputs.pin_motion_bounce_ms);
        let out_ringer = gen_optional_output(&gpio, config.enable_ringer, outputs.pin_ringer);
        let out_vibe = gen_optional_output(&gpio, config.enable_vibration, outputs.pin_vibrate);

        // Register pulse-dialing pins
        let (in_dial_switch, in_dial_pulse) = match phone_type {
            Rotary => {
                let pin_dial_pulse = inputs.pin_dial_pulse.expect("gpio.inputs.pin-dial-pulse is required for this phone type, but was not defined");
                let pin_dial_switch = inputs.pin_dial_pulse.expect("gpio.inputs.pin-dial-switch is required for this phone type, but was not defined");
                let in_dial_pulse = gen_required_soft_input(&gpio, pin_dial_pulse, inputs.pin_dial_pulse_bounce_ms);
                let in_dial_switch = gen_required_soft_input(&gpio, pin_dial_switch, inputs.pin_dial_switch_bounce_ms);
                (Some(in_dial_switch), Some(in_dial_pulse))
            },
            _ => (None, None)
        };

        // Register touch-tone dialing pins
        let (in_keypad_rows, out_keypad_cols) = match phone_type {
            TouchTone => {
                let pins_keypad_rows = inputs.pins_keypad_rows.expect("gpio.inputs.pins-keypad-rows is required for this phone type, but was not defined");
                let pins_keypad_cols = outputs.pins_keypad_cols.expect("gpio.outputs.pins-keypad-cols is required for this phone type, but was not defined");
                let in_keypad_rows: [SoftInputPin; 4] = [
                    gen_required_soft_input(&gpio, pins_keypad_rows[0], inputs.pins_keypad_rows_bounce_ms),
                    gen_required_soft_input(&gpio, pins_keypad_rows[1], inputs.pins_keypad_rows_bounce_ms),
                    gen_required_soft_input(&gpio, pins_keypad_rows[2], inputs.pins_keypad_rows_bounce_ms),
                    gen_required_soft_input(&gpio, pins_keypad_rows[3], inputs.pins_keypad_rows_bounce_ms)
                ];
                let out_keypad_cols: [OutputPin; 3] = [
                    gen_required_output(&gpio, pins_keypad_cols[0]),
                    gen_required_output(&gpio, pins_keypad_cols[1]),
                    gen_required_output(&gpio, pins_keypad_cols[2])
                ];
                (Some(in_keypad_rows), Some(out_keypad_cols))
            },
            _ => (None, None)
        };

        // Rotary dial state
        let dial_pulse_count = Arc::new(Mutex::new(0));
        let dial_switch_state = Arc::new(Mutex::new(true));

        GpioInterface {
            gpio,
            dial_pulse_count,
            dial_switch_state,
            in_hook,
            in_dial_switch,
            in_dial_pulse,
            in_motion,
            in_keypad_rows,
            out_keypad_cols,
            out_ringer,
            out_vibe,
            config: config.gpio
        }
    }
}

impl GpioInterface {
    fn listen(&mut self) -> Result<mpsc::Receiver<PhoneInputSignal>> {
        use PhoneInputSignal::*;
        let (tx, rx) = mpsc::channel();
        
        // On/Off-hook GPIO events
        let sender = tx.clone();
        self.in_hook.on_changed(move |state| {
            sender.send(HookState(state)).unwrap();
        })?;

        // Motion sensor
        let sender = tx.clone();
        if let Some(in_motion) = &mut self.in_motion {
            in_motion.on_changed(move |motion_detected| {
                if motion_detected {
                     sender.send(Motion).unwrap();
                }
            })?;
        }

        // TODO

        Ok(rx)
    }
}