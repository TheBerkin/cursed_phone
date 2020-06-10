#![allow(dead_code)]

use std::rc::Rc;
use std::cell::RefCell;
use std::{time, sync::mpsc, thread, io::{stdin, Read}};
use crate::config::*;
use crate::sound::*;

#[cfg(feature = "rpi")]
use crate::gpio::*;

#[derive(Copy, Clone, Debug)]
pub enum PhoneInputSignal {
    HookState(bool),
    Motion,
    Digit(char),
}

pub enum PhoneOutputSignal {
    Ring(bool), // TODO: Add cadence settings to ringer
    Vibrate { on: bool, duty_cycle: f32, time_seconds: f32 }
}

#[derive(Copy, Clone, Debug)]
pub enum PhoneType {
    Rotary,
    TouchTone,
    Other
}

pub enum PhoneState {
    Idle = 0,
    Dial = 1,
    PostDialDelay = 2,
    Ringback = 3,
    Connected = 4,
    Ringing = 5,
    BusyTone = 6
}

impl PhoneType {
    /// Converts a string to a `PhoneType`.
    /// Unsupported strings will return `Other`.
    fn from_name(name: &str) -> PhoneType {
        use PhoneType::*;
        match name {
            "rotary" => Rotary,
            "touchtone" => TouchTone,
            "other" | _ => Other
        }
    }
}

/// Provides I/O handling and state management for host phone peripherals.
pub struct PhoneEngine {
    phone_type: PhoneType,
    sound_engine: Rc<RefCell<SoundEngine>>,
    input_from_gpio: mpsc::Receiver<PhoneInputSignal>,
    output_to_pbx: RefCell<Option<Rc<mpsc::Sender<PhoneInputSignal>>>>,
    input_from_pbx: RefCell<Option<Rc<mpsc::Receiver<PhoneOutputSignal>>>>,
    dial_rest_state: bool,
    dial_pulse_state: bool,
    hook_state: bool,
    ring_state: bool,
    vibe_state: bool,
    #[cfg(feature = "rpi")]
    gpio: GpioInterface
}

impl PhoneEngine {
    /// Constructor for Phone on Raspberry Pi platforms.
    #[cfg(feature = "rpi")]
    pub fn new(config: &Rc<CursedConfig>, sound_engine: &Rc<RefCell<SoundEngine>>) -> Self {
        let phone_type = PhoneType::from_name(config.phone_type.as_str());
        let sound_engine = sound_engine.clone();
        let mut gpio = GpioInterface::new(phone_type, &config);
        let listener = gpio.listen().expect("Unable to initialize GPIO listener.");
        Self {
            phone_type,
            sound_engine,
            input_from_gpio: listener,
            dial_rest_state: true,
            dial_pulse_state: false,
            hook_state: true,
            ring_state: false,
            vibe_state: false,
            output_to_pbx: Default::default(),
            input_from_pbx: Default::default(),
            gpio
        }
    }

    /// Constructor for Phone on non-Pi platforms.
    #[cfg(not(feature = "rpi"))]
    pub fn new(config: &Rc<CursedConfig>, sound_engine: &Rc<RefCell<SoundEngine>>) -> Self {
        let phone_type = PhoneType::from_name(config.phone_type.as_str());
        let sound_engine = sound_engine.clone();
        // We won't use the JoinHandle here since it's frankly pretty useless in this case
        let (_, listener) = PhoneEngine::create_mock_input_thread();

        println!("Mock input is enabled. To send inputs, type a sequence of the following characters and press Enter:");
        println!("  - i: On-hook signal");
        println!("  - o: Off-hook signal");
        println!("  - m: Motion signal");
        println!("  - 0-9, A-D, #, *: Dial digit");

        Self {
            phone_type,
            sound_engine,
            input_from_gpio: listener,
            dial_rest_state: true,
            dial_pulse_state: false,
            hook_state: true,
            ring_state: false,
            vibe_state: false,
            output_to_pbx: Default::default(),
            input_from_pbx: Default::default()
        }
    }

    #[cfg(not(feature = "rpi"))]
    fn create_mock_input_thread() -> (thread::JoinHandle<()>, mpsc::Receiver<PhoneInputSignal>) {
        let (tx, rx) = mpsc::channel();
        let thread = thread::spawn(move || {
            let input = stdin();
            let mut reader = input.lock();
            let mut cbuf = [0u8];
            while let Ok(_) = reader.read(&mut cbuf) {
                use PhoneInputSignal::*;
                let signal: PhoneInputSignal = match (cbuf[0] as char).to_ascii_lowercase() {
                    'i' | 'I' => HookState(false),
                    'o' | 'O' => HookState(true),
                    'm' | 'M' => Motion,
                    digit @ '0'..='9' | digit @ 'a'..='d' | digit @ '*' | digit @ '#' => {
                        thread::sleep(time::Duration::from_millis(200));
                        Digit(digit.to_ascii_uppercase())
                    },
                    '-' => {
                        thread::sleep(time::Duration::from_millis(250));
                        continue;
                    },
                    '_' => {
                        thread::sleep(time::Duration::from_millis(750));
                        continue;
                    },
                    '.' => {
                        thread::sleep(time::Duration::from_millis(1000));
                        continue;
                    },
                    _ => continue
                };
                tx.send(signal).unwrap();
            }
        });
        (thread, rx)
    }
}

impl PhoneEngine {
    pub fn tick(&self) {
        // Process GPIO inputs
        if let Ok(signal) = self.input_from_gpio.try_recv() {
            use PhoneInputSignal::*;

            // Perform any additional processing here before passing on the signal
            match signal {
                HookState(on_hook) => {
                    #[cfg(not(feature = "rpi"))]
                    self.sound_engine.borrow().play(
                        if on_hook { "handset/hangup*" } else { "handset/pickup*" }, 
                        Channel::SignalOut, 
                        false, 
                        false, 
                        true, 
                        1.0, 
                        1.0);
                },
                Motion => {
                    println!("Detected motion!");
                },
                Digit(digit) => {
                    self.sound_engine.borrow().play_dtmf(digit, 0.1, 1.0);
                }
            }

            self.send_to_pbx(signal);
        }

        // Process GPIO outputs
        if let Some(input_from_pbx) = self.input_from_pbx.borrow().as_ref() {
            if let Ok(signal) = input_from_pbx.try_recv() {
                use PhoneOutputSignal::*;
                match signal {
                    Ring(on) => {
                        println!("Ringing = {}", on);
                        if on {
                            self.sound_engine.borrow().play("rings/ring_spkr_*", Channel::SignalOut, false, true, true, 1.0, 0.25);
                        } else {
                            self.sound_engine.borrow().stop(Channel::SignalOut)
                        }
                        // TODO: Pass ringing to GPIO
                    },
                    Vibrate { on, duty_cycle, time_seconds } => {
                        println!("Vibration = {}", on);
                        // TODO: Pass vibration to GPIO
                    }
                }
            }
        }
    }

    fn send_to_pbx(&self, input: PhoneInputSignal) -> bool {
        if let Some(tx) = self.output_to_pbx.borrow().as_ref() {
            tx.send(input).unwrap();
            return true;
        }
        false
    }

    /// Creates a messaging channel for the PBX to listen to input signals from the phone.
    pub fn gen_phone_output(&self) -> mpsc::Receiver<PhoneInputSignal> {
        let (tx_pbx, rx_input) = mpsc::channel();
        self.output_to_pbx.replace(Some(Rc::new(tx_pbx)));
        rx_input
    }

    pub fn listen_from_pbx(&self, input_from_pbx: mpsc::Receiver<PhoneOutputSignal>) {
        self.input_from_pbx.replace(Some(Rc::new(input_from_pbx)));
    }
}