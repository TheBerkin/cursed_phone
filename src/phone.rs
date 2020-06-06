#![allow(dead_code)]

use std::rc::Rc;
use std::cell::RefCell;
use std::{time, sync::mpsc, thread, io::{stdin, Read}};
use crate::config::*;
use crate::sound::SoundEngine;

#[cfg(feature = "rpi")]
use crate::gpio::*;

#[derive(Copy, Clone, Debug)]
pub enum PhoneInputSignal {
    HookState(bool),
    Motion,
    Digit(char),
}

#[derive(Copy, Clone, Debug)]
pub enum PhoneType {
    Rotary,
    TouchTone,
    Unknown // TODO: Rename to "Other"?
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
    sound_engine: Rc<RefCell<SoundEngine>>,
    listener: mpsc::Receiver<PhoneInputSignal>,
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
    pub fn new(config: &CursedConfig, sound_engine: &Rc<RefCell<SoundEngine>>) -> Self {
        let phone_type = PhoneType::from_name(config.phone_type.as_str());
        let sound_engine = sound_engine.clone();
        let mut gpio = GpioInterface::new(phone_type, &config);
        let listener = gpio.listen().expect("Unable to initialize GPIO listener.");

        Self {
            phone_type,
            sound_engine,
            listener,
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
    pub fn new(config: &CursedConfig, sound_engine: &Rc<RefCell<SoundEngine>>) -> Self {
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
            listener,
            on_hook: true,
            dial_resting: true,
            dial_pulse: false,
            ring_state: false,
            vibe_state: false,
            pdd: config.pdd,
            off_hook_delay: config.off_hook_delay
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
                    'i' | 'I' => HookState(true),
                    'o' | 'O' => HookState(false),
                    'm' | 'M' => Motion,
                    digit @ '0'..='9' | digit @ 'a'..='d' | digit @ '*' | digit @ '#' => {
                        thread::sleep(time::Duration::from_millis(150));
                        Digit(digit.to_ascii_uppercase())
                    },
                    ' ' => {
                        thread::sleep(time::Duration::from_millis(200));
                        continue;
                    },
                    '.' => {
                        thread::sleep(time::Duration::from_millis(250));
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
        if let Ok(signal) = self.listener.try_recv() {
            use PhoneInputSignal::*;
            match signal {
                HookState(on_hook) => {
                    println!("ON HOOK: {}", on_hook);
                    // TODO
                },
                Motion => todo!(),
                Digit(digit) => {
                    println!("DIALED: {}", digit);
                    self.sound_engine.borrow().play_dtmf(digit, 0.1, 1.0);
                }
            }
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