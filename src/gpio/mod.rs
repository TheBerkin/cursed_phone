#![cfg(feature = "rpi")]
#![allow(dead_code)]

mod debounce;
mod pull;

pub use debounce::*;
pub use pull::*;
use rppal::gpio::{Gpio, OutputPin, InputPin, Pin};

use std::{sync::{mpsc, Mutex, Arc, atomic::{AtomicBool, Ordering}}, collections::HashMap};
use std::time::{Instant, Duration};
use std::thread;
use std::rc::Rc;
use log::{info, warn};
use crate::config::*;
use crate::phone::*;


const KEYPAD_MIN_DIGIT_INTERVAL: Duration = Duration::from_millis(80);
const KEYPAD_ROW_BOUNCE: Duration = Duration::from_micros(850);
const KEYPAD_SCAN_INTERVAL: Duration = Duration::from_micros(1000);
const KEYPAD_COL_COUNT: usize = 3;
const KEYPAD_ROW_COUNT: usize = 4;
const KEYPAD_DIGITS: &[u8; KEYPAD_COL_COUNT * KEYPAD_ROW_COUNT] = b"123456789*0#";

/// Provides a general-purpose interface for accessing GPIO pins.
pub struct GpioInterface {
    pub gpio: Gpio,
    input_pins: HashMap<u8, SoftInputPin>,
    output_pins: HashMap<u8, OutputPin>,
}

impl GpioInterface {
    pub fn new() -> Result<Self, rppal::gpio::Error> {
        let gpio = Gpio::new()?;
        Ok(Self {
            gpio,
            input_pins: Default::default(),
            output_pins: Default::default(),
        })
    }

    pub fn register_input(&mut self, pin_id: u8, pull: Pull, bounce_time: Option<Duration>) -> Result<(), rppal::gpio::Error> {
        let pin = gen_input_pin(self.gpio.get(pin_id)?, pull).debounce(bounce_time.unwrap_or_default());
        self.input_pins.insert(pin_id, pin);
        Ok(())
    }

    pub fn register_output(&mut self, pin_id: u8) -> Result<(), rppal::gpio::Error> {
        let pin = self.gpio.get(pin_id)?.into_output();
        self.output_pins.insert(pin_id, pin);
        Ok(())
    }

    pub fn read_pin(&self, pin_id: u8) -> Option<bool> {
        if let Some(pin) = self.input_pins.get(&pin_id) {
            return Some(pin.is_high())
        }
        None
    }

    pub fn write_pin(&mut self, pin_id: u8, logic_level: bool) {
        if let Some(pin) = self.output_pins.get_mut(&pin_id) {
            if logic_level {
                pin.set_high()
            } else {
                pin.set_low()
            }
        }
    }

    pub fn set_pwm(&mut self, pin_id: u8, period: f64, pulse: f64) -> Result<(), rppal::gpio::Error> {
        if let Some(pin) = self.output_pins.get_mut(&pin_id) {
            pin.set_pwm(Duration::from_secs_f64(period), Duration::from_secs_f64(pulse))?;
        }
        Ok(())
    }

    pub fn clear_pwm(&mut self, pin_id: u8) -> Result<(), rppal::gpio::Error> {
        if let Some(pin) = self.output_pins.get_mut(&pin_id) {
            pin.clear_pwm()?;
        }
        Ok(())
    }

    pub fn unregister(&mut self, pin_id: u8) {
        self.input_pins.remove(&pin_id);
        self.output_pins.remove(&pin_id);
    }

    pub fn unregister_all(&mut self) {
        self.input_pins.clear();
        self.output_pins.clear();
    }
}

/// Provides an interface for phone-related GPIO pins.
/// This doesn't handle GPIO pins registered from Lua.
pub struct PhoneGpioInterface {
    gpio: Gpio,
    /// Pin for switch hook input.
    in_hook: SoftInputPin,
    /// Pin for dial switch input.
    in_dial_switch: Option<SoftInputPin>,
    /// Pin for dial pulse switch input.
    in_dial_pulse: Option<SoftInputPin>,
    /// Pins for keypad row inputs.
    in_keypad_rows: Option<[Arc<Mutex<SoftInputPin>>; KEYPAD_ROW_COUNT]>,
    /// Pins for coin trigger switch inputs.
    in_coin_triggers: Option<Vec<(u32, SoftInputPin)>>,
    /// Active state for coin trigger switch inputs.
    coin_trigger_active_state: bool,
    /// Pins for keypad column outputs.
    out_keypad_cols: Option<Arc<Mutex<[OutputPin; KEYPAD_COL_COUNT]>>>,
    /// Pin for ringer output.
    out_ringer: Option<Arc<Mutex<OutputPin>>>,
    /// Transmission channel for ringer control
    tx_ringer: Option<mpsc::Sender<Option<Arc<RingPattern>>>>,
    /// Copy of config used to initialize pins.
    config: Rc<CursedConfig>
}

pub fn gen_input_pin(pin: Pin, pull: Pull) -> InputPin {
    match pull {
        Pull::Up => pin.into_input_pullup(),
        Pull::Down => pin.into_input_pulldown(),
        Pull::None => {
            warn!("Input pin {} is floating; internal pull resistors are disabled.", pin.pin());
            pin.into_input()
        }
    }
}

fn gen_optional_soft_input_from(gpio: &Gpio, enable: Option<bool>, input_config: &Option<InputPinConfig>) -> Option<SoftInputPin> {
    if enable.unwrap_or(false) {
        if let Some(input_config) = input_config {
            let pin = gpio.get(input_config.pin).unwrap();
            let input = gen_input_pin(pin, Pull::from(&input_config.pull));
            let soft_input = input.debounce(Duration::from_millis(input_config.bounce_ms.unwrap_or(0)));
            return Some(soft_input);
        }
    }
    None
}

fn gen_required_soft_input_from(gpio: &Gpio, input_config: &InputPinConfig) -> SoftInputPin {
    let pin = gpio.get(input_config.pin).unwrap();
    let raw_input = gen_input_pin(pin, Pull::from(&input_config.pull));
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
    gen_input_pin(gpio.get(pin).unwrap(), pull).debounce(debounce.unwrap_or_default())
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

impl PhoneGpioInterface {
    pub fn new(config: &Rc<CursedConfig>) -> PhoneGpioInterface {
        let gpio = Gpio::new().expect("Unable to initialize GPIO interface");
        let inputs = &config.gpio.inputs;
        let outputs = &config.gpio.outputs;
        let mut tx_ringer = None;

        // Register standard GPIO pins
        let in_hook = gen_required_soft_input_from(&gpio, &inputs.switchhook);

        let out_ringer = gen_optional_output(&gpio, config.ringer_enabled, outputs.pin_ringer)
            .map(|o| Arc::new(Mutex::new(o)));

        // Register pulse-dialing pins
        let (in_dial_switch, in_dial_pulse) = if config.rotary.enabled {
            let dial_pulse = config.rotary.input_pulse.as_ref().expect("missing configuration for rotary pulse input");
            let dial_switch = config.rotary.input_rest.as_ref().expect("missing configuration for rotary rest input");
            let in_dial_pulse = gen_required_soft_input_from(&gpio, dial_pulse);
            let in_dial_switch = gen_required_soft_input_from(&gpio, dial_switch);
            (Some(in_dial_switch), Some(in_dial_pulse))
        } else {
            (None, None)
        };

        // Register touch-tone dialing pins
        let (in_keypad_rows, out_keypad_cols) = if config.keypad.enabled {
            let pins_keypad_rows = config.keypad.input_rows.expect("missing configuration for keypad row inputs");
                let pins_keypad_cols = config.keypad.output_cols.expect("missing configuration for keypad column outputs");
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
        } else {
            (None, None)
        };

        // Ringer thread
        if config.ringer_enabled.unwrap_or(false) {
            let (tx, rx) = mpsc::channel::<Option<Arc<RingPattern>>>();
            tx_ringer = Some(tx);
            let ringer: Arc<Mutex<OutputPin>> = Arc::clone(out_ringer.as_ref().unwrap());

            thread::spawn(move || {
                const RINGER_FREQ_DEFAULT: f64 = 20.0;
                const RINGER_DUTY_CYCLE_DEFAULT: f64 = 0.5;
                let mut next_pattern: Option<Arc<RingPattern>> = None;

                'poll_pattern: loop {
                    // Stop any ringing that was interrupted
                    {
                        let mut ringer = ringer.lock().unwrap();
                        ringer.clear_pwm().unwrap();
                        ringer.set_low();
                    }

                    'read_pattern: while let Some(pattern) = next_pattern.take().or_else(|| rx.recv().ok().flatten()) {
                        loop {
                            // Play the ring pattern     
                            for step in pattern.components.iter() {
                                macro_rules! ringer_wait {
                                    ($dur:expr) => {
                                        match rx.recv_timeout($dur) {
                                            Ok(Some(new_pattern)) => {
                                                next_pattern = Some(new_pattern);
                                                continue 'read_pattern
                                            },
                                            Ok(None) => continue 'poll_pattern,
                                            Err(_) => {}
                                        }
                                    }
                                }
                                match step {
                                    RingPatternComponent::RingWithCycle { high, low, duration } => {
                                        let cycle_length = *high + *low;
                                        let mut ringer = ringer.lock().unwrap();
                                        ringer.set_pwm(cycle_length, *high).unwrap();
                                        ringer_wait!(*duration)
                                    },
                                    RingPatternComponent::RingWithFrequency { frequency, duration } => {
                                        let mut ringer = ringer.lock().unwrap();
                                        ringer.set_pwm_frequency(*frequency, 0.5).unwrap();
                                        ringer_wait!(*duration)
                                    },
                                    RingPatternComponent::Ring(duration) => {
                                        let mut ringer = ringer.lock().unwrap();
                                        ringer.set_pwm_frequency(RINGER_FREQ_DEFAULT, RINGER_DUTY_CYCLE_DEFAULT).unwrap();
                                        ringer_wait!(*duration)
                                    },
                                    RingPatternComponent::Low(duration) => {
                                        let mut ringer = ringer.lock().unwrap();
                                        ringer.clear_pwm().unwrap();
                                        ringer.set_low();
                                        ringer_wait!(*duration);
                                    },
                                    RingPatternComponent::High(duration) => {
                                        let mut ringer = ringer.lock().unwrap();
                                        ringer.clear_pwm().unwrap();
                                        ringer.set_high();
                                        ringer_wait!(*duration);
                                    },
                                    RingPatternComponent::End => break 'read_pattern
                                }
                            }
                        }
                    }
                }
            });
        }

        // Register coin trigger pins
        let mut coin_trigger_active_state = false;
        let in_coin_triggers = if config.payphone.enabled {
            config.payphone.coin_values.as_ref().map(|coin_values| {
                if coin_values.is_empty() {
                    warn!("no payphone coin values specified; disabling payphone features.");
                    return None
                }

                let coin_trigger_pins = config.payphone.coin_input_pins.as_ref()
                    .expect("missing configuration for payphone coin trigger pins");
                let coin_trigger_bounce_ms = config.payphone.coin_input_bounce_ms.as_ref()
                    .expect("missing configuration for payphone coin trigger debounce timings");

                if coin_trigger_pins.len() != coin_values.len() {
                    warn!("payphone trigger pin count does not match coin value count!");
                    return None
                }

                if coin_trigger_bounce_ms.len() != coin_values.len() {
                    warn!("payphone trigger pin count does not match coin value count!");
                    return None
                }

                let pull = Pull::from(&config.payphone.coin_input_pull);

                coin_trigger_active_state = match pull {
                    Pull::Down => true,
                    Pull::Up => false,
                    _ => true
                };

                let in_coin_triggers: Vec<(u32, SoftInputPin)> = coin_trigger_pins
                    .iter()
                    .zip(coin_trigger_bounce_ms.iter().map(|ms| Duration::from_millis(*ms)))
                    .zip(coin_values.iter())
                    .map(|((pin, bounce), cents)| (*cents, gen_required_soft_input(&gpio, *pin, Some(bounce), pull)))
                    .collect();

                info!("Coin triggers initialized ({}).", in_coin_triggers.len());

                return Some(in_coin_triggers)
            }).flatten()
        } else {
            None
        };

        PhoneGpioInterface {
            gpio,
            in_hook,
            in_dial_switch,
            in_dial_pulse,
            in_keypad_rows,
            in_coin_triggers,
            coin_trigger_active_state,
            out_keypad_cols,
            out_ringer,
            tx_ringer,
            config: Rc::clone(config)
        }
    }
}

impl PhoneGpioInterface {
    pub fn listen(&mut self) -> Result<mpsc::Receiver<PhoneInputSignal>, rppal::gpio::Error> {
        let (tx, rx) = mpsc::channel();
        
        // On/Off-hook GPIO events
        let sender = tx.clone();
        self.in_hook.set_on_changed(move |state| {
            sender.send(PhoneInputSignal::HookState(state)).unwrap();
        });

        // Rotary dial rest switch
        if let Some(in_dial_switch) = &mut self.in_dial_switch {
            let sender = tx.clone();
            in_dial_switch.set_on_changed(move |dial_resting| {
                sender.send(PhoneInputSignal::RotaryDialRest(dial_resting)).unwrap();
            });
        }

        // Rotary dial pulse switch
        if let Some(in_dial_pulse) = &mut self.in_dial_pulse {
            let sender = tx.clone();
            in_dial_pulse.set_on_changed(move |dial_pulse_state| {
                // We're only interested in the closed state of the pulse,
                // as the full pulse is implied to have happened for this state to be reached.
                if dial_pulse_state {
                    sender.send(PhoneInputSignal::RotaryDialPulse).unwrap();
                }
            });
        }

        // Touch-tone keypad
        if let (Some(rows), Some(cols)) 
        = (&mut self.in_keypad_rows, &mut self.out_keypad_cols) {
            // Set the cols initially high
            let mut cols_lock = cols.lock().unwrap();
            for j in 0..KEYPAD_COL_COUNT {
                cols_lock[j].set_high();
            }

            let (tx_keypad, rx_keypad) = mpsc::channel();
            let cols = Arc::clone(cols);
            let sender = tx.clone();
            let suppress_row_events = Arc::new(AtomicBool::new(false));            
            let suppress_row_events_cl = Arc::clone(&suppress_row_events);

            // Create keypad input handler thread
            thread::spawn(move || {
                let mut last_digit_time = Instant::now();

                while let Ok((row_index, row_high)) = rx_keypad.recv() {
                    let mut cols = cols.lock().unwrap();
                    let current_press_time = Instant::now();
                    if row_high && current_press_time.checked_duration_since(last_digit_time).unwrap_or_default() >= KEYPAD_MIN_DIGIT_INTERVAL {
                        // Turn off each col until row turns off
                        for col_index in 0..KEYPAD_COL_COUNT {
                            cols[col_index].set_low();
                            if rx_keypad.recv_timeout(KEYPAD_SCAN_INTERVAL) == Ok((row_index, false)) {
                                // Calculate digit from row/col indices
                                let digit_index = row_index * KEYPAD_COL_COUNT + col_index;
                                let digit = KEYPAD_DIGITS[digit_index] as char;
                                sender.send(PhoneInputSignal::Digit(digit)).unwrap();
                                last_digit_time = current_press_time;
                                break;
                            }
                        }

                        // Turn cols back on
                        suppress_row_events_cl.store(true, Ordering::SeqCst);
                        for col_index in 0..KEYPAD_COL_COUNT {
                            cols[col_index].set_high();
                        }
                        thread::sleep(KEYPAD_SCAN_INTERVAL);
                        suppress_row_events_cl.store(false, Ordering::SeqCst);
                    }
                }
            });

            // Create input handler for each keypad row
            for i in 0..KEYPAD_ROW_COUNT {
                let tx_keypad = tx_keypad.clone();
                let suppress_row_events = Arc::clone(&suppress_row_events);
                rows[i].lock().unwrap().set_on_changed(move |state| {
                    if suppress_row_events.load(Ordering::SeqCst) { return }
                    if state {
                        tx_keypad.send((i, true)).expect("unable to communicate with keypad input handler thread");
                    } else {
                        tx_keypad.send((i, false)).expect("unable to communicate with keypad input handler thread");
                    }
                });
            }
        }

        // Coin mechanism
        if let Some(in_coin_triggers) = self.in_coin_triggers.as_mut() {
            let coin_triggers_iter = in_coin_triggers.iter_mut();
            for (cents, input) in coin_triggers_iter {
                let cents = *cents;
                let active_state = self.coin_trigger_active_state;
                let sender = tx.clone();
                input.set_on_changed(move |state| {
                    if state == active_state {
                        sender.send(PhoneInputSignal::Coin(cents)).unwrap();
                    }
                });
            }
        }

        info!("GPIO peripherals initialized.");
        Ok(rx)
    }

    pub fn tx_ringer(&self) -> Option<mpsc::Sender<Option<Arc<RingPattern>>>> {
        if let Some(tx) = &self.tx_ringer {
            return Some(tx.clone())
        }
        None
    }
}