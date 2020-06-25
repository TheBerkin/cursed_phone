#![cfg(feature = "rpi")]
#![allow(dead_code)]

use std::sync::{mpsc, Mutex, Arc, atomic::{AtomicBool, Ordering}};
use std::time::{Instant, Duration};
use std::thread;
use std::rc::Rc;
use log::{info, trace};
use rppal::gpio::*;
use crate::config::*;
use crate::phone::*;

const KEYPAD_ROW_BOUNCE: Duration = Duration::from_micros(50);
const KEYPAD_SCAN_INTERVAL: Duration = Duration::from_micros(250);
const KEYPAD_COL_COUNT: usize = 3;
const KEYPAD_ROW_COUNT: usize = 4;
const KEYPAD_DIGITS: &[u8; KEYPAD_COL_COUNT * KEYPAD_ROW_COUNT] = b"123456789*0#";

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

    fn is_debouncing(&self) -> bool;

    fn set_bounce_time(&mut self, time: Duration);
}

/// Simple wrapper around `rppal::gpio::pin::InputPin` to add debouncing.
struct SoftInputPin {
    pin: Arc<Mutex<InputPin>>,
    state: Arc<Mutex<SoftInputState>>,
    debounce_flag: Arc<AtomicBool>
}

struct SoftInputState {
    bounce_time: Duration,
    last_changed: Instant,
    last_value: bool
}

impl SoftInputState {
    fn change_last_value(&mut self, new_value: bool) {
        self.last_changed = Instant::now();
        self.last_value = new_value;
    }
}

impl SoftInputPin {
    fn new(mut pin: InputPin, bounce_time: Duration) -> Self {
        pin.set_interrupt(Trigger::Both).unwrap();
        let last_changed = Instant::now();
        let last_value = pin.is_high();

        let state = SoftInputState {
            last_changed,
            bounce_time,
            last_value
        };

        Self {
            pin: Arc::new(Mutex::new(pin)),
            state: Arc::new(Mutex::new(state)),
            debounce_flag: Arc::new(AtomicBool::new(false))
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
        let state = Arc::clone(&self.state);
        let pin = Arc::clone(&self.pin);
        let debounce_flag = Arc::clone(&self.debounce_flag);
        self.pin.lock().unwrap().set_async_interrupt(Trigger::Both, move |level| {
            // If the pin is currently debouncing, ignore this event entirely.
            if debounce_flag.load(Ordering::SeqCst) {
                return;
            }
            
            // Pin value at time of event
            let mut new_value = level == Level::High;

            // Acquire debounce state and pin
            let mut state = state.lock().unwrap();
            let pin = pin.lock().unwrap();
            
            // Ignore this event if the state hasn't changed
            if new_value != state.last_value {
                // Enable debounce flag
                debounce_flag.store(true, Ordering::SeqCst);

                // Inform user of value change
                state.change_last_value(new_value);
                callback(new_value);

                // Sleep for bounce time
                thread::sleep(state.bounce_time);

                // Check if input has changed since debounce and inform user if so
                new_value = pin.is_high();
                if new_value != state.last_value {
                    state.change_last_value(new_value);
                    callback(new_value);
                }

                // End debounce
                debounce_flag.store(false, Ordering::SeqCst);
            }
        })
    }

    #[inline]
    fn is_debouncing(&self) -> bool {
        self.debounce_flag.load(Ordering::SeqCst)
    }

    fn is_high(&self) -> bool {
        let mut state = self.state.lock().unwrap();
        if state.last_changed.elapsed() < state.bounce_time {
            return state.last_value;
        }
        let new_state = self.pin.lock().unwrap().is_high();
        state.last_value = new_state;
        state.last_changed = Instant::now();
        new_state
    }

    #[inline]
    fn is_low(&self) -> bool {
        !self.is_high()
    }

    fn set_bounce_time(&mut self, time: Duration) {
        let mut state = self.state.lock().unwrap();
        state.bounce_time = time;
    }
}

enum Pull {
    None,
    Up,
    Down
}

/// Provides an interface for phone-related GPIO pins.
pub struct GpioInterface {
    gpio: Gpio,
    /// Pin for switch hook input.
    in_hook: SoftInputPin,
    /// Pin for dial switch input.
    in_dial_switch: Option<SoftInputPin>,
    /// Pin for dial pulse switch input.
    in_dial_pulse: Option<SoftInputPin>,
    /// Pin for motion detector input.
    in_motion: Option<SoftInputPin>,
    /// Pins for keypad row inputs.
    in_keypad_rows: Option<[Arc<Mutex<SoftInputPin>>; KEYPAD_ROW_COUNT]>,
    /// Pins for keypad column outputs.
    out_keypad_cols: Option<Arc<Mutex<[OutputPin; KEYPAD_COL_COUNT]>>>,
    /// Pin for ringer output.
    out_ringer: Option<Arc<Mutex<OutputPin>>>,
    /// Pin for vibration motor output.
    out_vibe: Option<Arc<Mutex<OutputPin>>>,
    /// Transmission channel for ringer control
    tx_ringer: Option<mpsc::Sender<bool>>, // TODO: Pass cadence data to ringer thread
    /// Copy of config used to initialize pins.
    config: Rc<CursedConfig>
}

fn gen_optional_soft_input_from(gpio: &Gpio, enable: Option<bool>, input_config: &Option<InputPinConfig>) -> Option<SoftInputPin> {
    if enable.unwrap_or(false) {
        if let Some(input) = input_config {
            let raw_pin = gpio.get(input.pin).unwrap();
            let raw_input = if let Some(pull_name) = &input.pull {
                match pull_name.to_ascii_lowercase().as_str() {
                    "up" => raw_pin.into_input_pullup(),
                    "down" => raw_pin.into_input_pulldown(),
                    "none" | _ => raw_pin.into_input()
                }
            } else {
                raw_pin.into_input()
            };
            let soft_input = raw_input.debounce(Duration::from_millis(input.bounce_ms.unwrap_or(0)));
            return Some(soft_input);
        }
    }
    None
}

fn gen_required_soft_input_from(gpio: &Gpio, input_config: &InputPinConfig) -> SoftInputPin {
    let raw_pin = gpio.get(input_config.pin).unwrap();
    let raw_input = if let Some(pull_name) = &input_config.pull {
        match pull_name.to_ascii_lowercase().as_str() {
            "up" => raw_pin.into_input_pullup(),
            "down" => raw_pin.into_input_pulldown(),
            "none" | _ => raw_pin.into_input()
        }
    } else {
        raw_pin.into_input()
    };
    let soft_input = raw_input.debounce(Duration::from_millis(input_config.bounce_ms.unwrap_or(0)));
    soft_input
}

fn gen_optional_soft_input(gpio: &Gpio, enable: Option<bool>, pin: Option<u8>, debounce: Option<Duration>) -> Option<SoftInputPin> {
    if enable.unwrap_or(false) {
        if let Some(pin) = pin {
            let input = gpio.get(pin).unwrap()
                .into_input_pullup()
                .debounce(debounce.unwrap_or_default());
            return Some(input);
        }
    }
    None
}

fn gen_required_soft_input(gpio: &Gpio, pin: u8, debounce: Option<Duration>, pull: Pull) -> SoftInputPin {
    match pull {
        Pull::None => gpio.get(pin).unwrap().into_input(),
        Pull::Up => gpio.get(pin).unwrap().into_input_pullup(),
        Pull::Down => gpio.get(pin).unwrap().into_input_pulldown()
    }
    .debounce(debounce.unwrap_or_default())
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
    pub fn new(phone_type: PhoneType, config: &Rc<CursedConfig>) -> GpioInterface {
        use PhoneType::*;
        let gpio = Gpio::new().expect("Unable to initialize GPIO interface");
        let inputs = &config.gpio.inputs;
        let outputs = &config.gpio.outputs;

        // Register standard GPIO pins
        let in_hook = gen_required_soft_input_from(&gpio, &inputs.hook);
        let in_motion = gen_optional_soft_input_from(&gpio, config.enable_motion_sensor, &inputs.motion);

        let out_ringer = if let Some(output) = gen_optional_output(&gpio, config.enable_ringer, outputs.pin_ringer) {
            Some(Arc::new(Mutex::new(output)))
        } else {
            None
        };

        let out_vibe = if let Some(output) = gen_optional_output(&gpio, config.enable_vibration, outputs.pin_vibrate) {
            Some(Arc::new(Mutex::new(output)))
        } else {
            None
        };

        // Register pulse-dialing pins
        let (in_dial_switch, in_dial_pulse) = match phone_type {
            Rotary => {
                let dial_pulse = inputs.dial_pulse.as_ref().expect("gpio.inputs.pin-dial-pulse is required for this phone type, but was not defined");
                let dial_switch = inputs.dial_switch.as_ref().expect("gpio.inputs.pin-dial-switch is required for this phone type, but was not defined");
                let in_dial_pulse = gen_required_soft_input_from(&gpio, dial_pulse);
                let in_dial_switch = gen_required_soft_input_from(&gpio, dial_switch);
                (Some(in_dial_switch), Some(in_dial_pulse))
            },
            _ => (None, None)
        };

        // Register touch-tone dialing pins
        let (in_keypad_rows, out_keypad_cols) = match phone_type {
            TouchTone | Payphone => {
                let pins_keypad_rows = inputs.pins_keypad_rows.expect("gpio.inputs.pins-keypad-rows is required for this phone type, but was not defined");
                let pins_keypad_cols = outputs.pins_keypad_cols.expect("gpio.outputs.pins-keypad-cols is required for this phone type, but was not defined");
                let in_keypad_rows = [
                    Arc::new(Mutex::new(gen_required_soft_input(&gpio, pins_keypad_rows[0], Some(KEYPAD_ROW_BOUNCE), Pull::Down))),
                    Arc::new(Mutex::new(gen_required_soft_input(&gpio, pins_keypad_rows[1], Some(KEYPAD_ROW_BOUNCE), Pull::Down))),
                    Arc::new(Mutex::new(gen_required_soft_input(&gpio, pins_keypad_rows[2], Some(KEYPAD_ROW_BOUNCE), Pull::Down))),
                    Arc::new(Mutex::new(gen_required_soft_input(&gpio, pins_keypad_rows[3], Some(KEYPAD_ROW_BOUNCE), Pull::Down))),
                ];
                let out_keypad_cols = Arc::new(Mutex::new([
                    gen_required_output(&gpio, pins_keypad_cols[0]),
                    gen_required_output(&gpio, pins_keypad_cols[1]),
                    gen_required_output(&gpio, pins_keypad_cols[2]),
                ]));
                (Some(in_keypad_rows), Some(out_keypad_cols))
            },
            _ => (None, None)
        };

        // Ringer fields
        let mut tx_ringer = None;
        if config.enable_ringer.unwrap_or(true) {
            let (tx, rx) = mpsc::channel();
            tx_ringer = Some(tx);
            let ringer: Arc<Mutex<OutputPin>> = Arc::clone(out_ringer.as_ref().unwrap());
            thread::spawn(move || {
                const RINGER_CADENCE: (f64, f64) = (2.0, 4.0);
                const RINGER_FREQ: f64 = 20.0;
                const RINGER_DUTY_CYCLE: f64 = 0.5;
                let ring_on_time = Duration::from_secs_f64(RINGER_CADENCE.0);
                let ring_off_time = Duration::from_secs_f64(RINGER_CADENCE.1);

                loop {
                    // Stop any ringing that was interrupted
                    ringer.lock().unwrap().clear_pwm().unwrap();

                    'ring_check: while let Ok(true) = rx.recv() {
                        loop {
                            let phase_start = Instant::now();

                            if let Ok(false) = rx.try_recv() {
                                break 'ring_check;
                            }

                            // Start ringing
                            ringer.lock().unwrap().set_pwm_frequency(RINGER_FREQ, 0.5).unwrap();

                            // Wait for on-time or cancel signal
                            while phase_start.elapsed() < ring_on_time {
                                if let Ok(false) = rx.try_recv() {
                                    break 'ring_check;
                                }
                            }

                            let phase_start = Instant::now();

                            // Stop ringing
                            ringer.lock().unwrap().clear_pwm().unwrap();

                            if let Ok(false) = rx.try_recv() {
                                break 'ring_check;
                            }

                            // Wait for off-time or cancel signal
                            while phase_start.elapsed() < ring_off_time {
                                if let Ok(false) = rx.try_recv() {
                                    break 'ring_check;
                                }
                            }
                        }
                    }
                }
            });
        }

        // Vibration fields
        // TODO: Set up vibration output
        

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
            tx_ringer,
            config: Rc::clone(config)
        }
    }
}

impl GpioInterface {
    pub fn listen(&mut self) -> Result<mpsc::Receiver<PhoneInputSignal>> {
        let (tx, rx) = mpsc::channel();
        
        // On/Off-hook GPIO events
        let sender = tx.clone();
        self.in_hook.on_changed(move |state| {
            sender.send(PhoneInputSignal::HookState(state)).unwrap();
        })?;

        // Motion sensor
        if let Some(in_motion) = &mut self.in_motion {
            let sender = tx.clone();
            in_motion.on_changed(move |motion_detected| {
                if motion_detected {
                     sender.send(PhoneInputSignal::Motion).unwrap();
                }
            })?;
        }

        // Rotary dial rest switch
        if let Some(in_dial_switch) = &mut self.in_dial_switch {
            let sender = tx.clone();
            in_dial_switch.on_changed(move |dial_resting| {
                sender.send(PhoneInputSignal::RotaryDialRest(dial_resting)).unwrap();
            })?;
        }

        // Rotary dial pulse switch
        if let Some(in_dial_pulse) = &mut self.in_dial_pulse {
            let sender = tx.clone();
            in_dial_pulse.on_changed(move |dial_pulse_state| {
                // We're only interested in the closed state of the pulse,
                // as the full pulse is implied to have happened for this state to be reached.
                if dial_pulse_state {
                    sender.send(PhoneInputSignal::RotaryDialPulse).unwrap();
                }
            })?;
        }

        // Touch-tone keypad
        if let (Some(rows), Some(cols)) 
        = (&mut self.in_keypad_rows, &mut self.out_keypad_cols) {

            // Set the cols initially high
            let mut cols_lock = cols.lock().unwrap();
            for j in 0..KEYPAD_COL_COUNT {
                cols_lock[j].set_high();
            }

            // Create input handler for each keypad row
            for i in 0..KEYPAD_ROW_COUNT {
                let sender = tx.clone();
                let cols = Arc::clone(cols);
                let row = Arc::downgrade(&Arc::clone(&rows[i]));
                rows[i].lock().unwrap().on_changed(move |state| {
                    let mut cols_lock = cols.lock().unwrap();
                    if state {
                        trace!("[Keypad] Row {} is high", i + 1);
                        // Turn off each col in series until the row turns off
                        for j in 0..KEYPAD_COL_COUNT {
                            cols_lock[j].set_low();
                            thread::sleep(KEYPAD_SCAN_INTERVAL);
                            if let Some(row) = row.upgrade() {
                                if row.lock().unwrap().is_low() {
                                    let digit = KEYPAD_DIGITS[i * KEYPAD_COL_COUNT + j] as char;
                                    sender.send(PhoneInputSignal::Digit(digit)).unwrap();
                                    break;
                                }
                            }
                        }
                    } else {
                        trace!("[Keypad] Row {} is low", i + 1);
                        // Turn the cols back on after reading the digit
                        for j in 0..KEYPAD_COL_COUNT {
                            cols_lock[j].set_high();
                        }
                    }
                }).unwrap();
            }
        }

        info!("GPIO peripherals initialized.");

        Ok(rx)
    }

    pub fn tx_ringer(&self) -> Option<mpsc::Sender<bool>> {
        if let Some(tx) = &self.tx_ringer {
            return Some(tx.clone())
        }
        None
    }
}