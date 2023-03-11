#![allow(dead_code)]
#![allow(unused_imports)]

use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Arc;
use std::{time, sync::mpsc, thread, io::{stdin, Read}};
use log::{info, trace, warn};
use logos::{Logos, Lexer};
use mlua::prelude::LuaUserData;
use crate::config::*;
use crate::sound::*;


#[cfg(feature = "rpi")]
use crate::gpio::*;
use time::Duration;

/// Represents user signals produced by interacting with the physical phone interface.
#[derive(Copy, Clone, Debug)]
pub enum PhoneInputSignal {
    HookState(bool),
    RotaryDialRest(bool),
    RotaryDialPulse,
    Coin(u32),
    Digit(char),
}

/// Represents signals produced by the phone.
pub enum PhoneOutputSignal {
    Ring(Option<Arc<RingPattern>>),
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

#[derive(Debug, Clone)]
pub struct RingPattern {
    pub components: Vec<RingPatternComponent>
}

#[derive(Debug, Clone)]
pub struct LuaRingPattern(pub Arc<RingPattern>);

impl LuaUserData for LuaRingPattern {}

impl RingPattern {
    pub fn try_parse(expr: &str) -> Option<Self> {
        use RingPatternToken::*;
        let mut components = vec![];
        let mut lex = RingPatternToken::lexer(expr);

        while let Some(token) = lex.next() {
            match token {
                RingPatternToken::KwCycle => {
                    if let (Number(h), Comma, Number(l), Comma, Number(t)) = (lex.next()?, lex.next()?, lex.next()?, lex.next()?, lex.next()?) {
                        components.push(RingPatternComponent::RingWithCycle { 
                            high: Duration::try_from_secs_f64(h / 1000.0).unwrap_or_default(), 
                            low: Duration::try_from_secs_f64(l / 1000.0).unwrap_or_default(), 
                            duration: Duration::try_from_secs_f64(t / 1000.0).unwrap_or_default(),
                        })
                    } else {
                        return None
                    }
                },
                RingPatternToken::KwFrequency => {
                    if let (Number(hz), Comma, Number(ms)) = (lex.next()?, lex.next()?, lex.next()?) {
                        components.push(RingPatternComponent::RingWithFrequency { 
                            frequency: hz, 
                            duration: Duration::try_from_secs_f64(ms / 1000.0).unwrap_or_default() 
                        })
                    } else {
                        return None
                    }
                },
                RingPatternToken::KwRing => {
                    if let Number(ms) = lex.next()? {
                        components.push(RingPatternComponent::Ring(Duration::try_from_secs_f64(ms / 1000.0).unwrap_or_default()))
                    }
                },
                RingPatternToken::KwLow => {
                    if let Some(Number(ms)) = lex.next() {
                        components.push(RingPatternComponent::Low(Duration::try_from_secs_f64(ms / 1000.0).unwrap_or_default()));
                    } else {
                        return None
                    }
                },
                RingPatternToken::KwHigh => {
                    if let Some(Number(ms)) = lex.next() {
                        components.push(RingPatternComponent::High(Duration::try_from_secs_f64(ms / 1000.0).unwrap_or_default()));
                    } else {
                        return None
                    }
                },
                _ => return None
            }
        }

        Some(Self {
            components
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum RingPatternComponent {
    RingWithCycle { high: Duration, low: Duration, duration: Duration },
    RingWithFrequency { frequency: f64, duration: Duration },
    Ring(Duration),
    Low(Duration),
    High(Duration),
    End,
}

#[derive(Logos, Debug, PartialEq)]
enum RingPatternToken {

    #[token("C")]
    KwCycle,
    #[token("R")]
    KwFrequency,
    #[token("Q")]
    KwRing,
    #[token("L")]
    KwLow,
    #[token("H")]
    KwHigh,
    #[token("$")]
    KwEnd,
    #[token(",")]
    Comma,
    #[regex(r"([0-9]+(\.[0-9]+)?|\.[0-9]+)", parse_ring_pattern_duration)]
    Number(f64),
    #[error]
    #[regex(r"[\t\r\n\f ]+", logos::skip)]
    Invalid,
}

fn parse_ring_pattern_duration(lex: &mut Lexer<RingPatternToken>) -> Option<f64> {
    let slice = lex.slice();
    let n: f64 = slice.parse().ok()?;
    Some(n)
}

/// Provides I/O handling and state management for host phone peripherals.
pub struct PhoneEngine {
    dtmf_tone_duration: Duration,
    sound_engine: Rc<RefCell<SoundEngine>>,
    rx_gpio: mpsc::Receiver<PhoneInputSignal>,
    rx_engine: RefCell<Option<Rc<mpsc::Receiver<PhoneOutputSignal>>>>,
    tx_engine: RefCell<Option<Rc<mpsc::Sender<PhoneInputSignal>>>>,
    tx_ringer: Option<mpsc::Sender<Option<Arc<RingPattern>>>>,
    dial_rest_state: bool,
    dial_pulse_state: bool,
    hook_state: bool,
    ring_state: bool,
    default_ring_pattern: RingPattern,
    #[cfg(feature = "rpi")]
    gpio: PhoneGpioInterface
}

impl PhoneEngine {
    /// Constructor for Phone on Raspberry Pi platforms.
    #[cfg(feature = "rpi")]
    pub fn new(config: &Rc<CursedConfig>, sound_engine: &Rc<RefCell<SoundEngine>>) -> Self {
        let sound_engine = sound_engine.clone();
        let mut gpio = PhoneGpioInterface::new(&config);
        let listener = gpio.listen().expect("Unable to initialize GPIO listener.");
        let tx_ringer = gpio.tx_ringer();
        Self {
            sound_engine,
            rx_gpio: listener,
            dial_rest_state: true,
            dial_pulse_state: false,
            hook_state: true,
            ring_state: false,
            dtmf_tone_duration: Duration::from_millis(config.sound.dtmf_tone_duration_ms),
            tx_engine: Default::default(),
            rx_engine: Default::default(),
            default_ring_pattern: RingPattern::try_parse(config.default_ring_pattern.as_str()).unwrap_or_else(|| {
                warn!("Unable to read default ring pattern from config. Using fallback pattern.");
                RingPattern {
                    components: vec![RingPatternComponent::Ring(Duration::from_secs(2)), RingPatternComponent::Low(Duration::from_secs(4))]
                }
            }),
            tx_ringer,
            gpio
        }
    }

    /// Constructor for Phone on non-Pi platforms.
    #[cfg(not(feature = "rpi"))]
    pub fn new(config: &Rc<CursedConfig>, sound_engine: &Rc<RefCell<SoundEngine>>) -> Self {
        use log::warn;
        let sound_engine = sound_engine.clone();
        // We won't use the JoinHandle here since it's frankly pretty useless in this case
        let (_, listener) = PhoneEngine::create_mock_input_thread();

        info!("Mock input is enabled. To send inputs, type a sequence of the following characters and press Enter:");
        info!("  - i: Off-hook signal");
        info!("  - o: On-hook signal");
        info!("  - w/r: Dial rest open/close");
        info!("  - e: Rotary dial pulse (full cycle)");
        info!("  - f/g/h/j: Insert 1¢/5¢/10¢/25¢");
        info!("  - 0-9, A-D, #, *: Dial digit");

        Self {
            sound_engine,
            rx_gpio: listener,
            dial_rest_state: true,
            dial_pulse_state: false,
            hook_state: true,
            ring_state: false,
            tx_ringer: None,
            dtmf_tone_duration: Duration::from_millis(config.sound.dtmf_tone_duration_ms),
            tx_engine: Default::default(),
            rx_engine: Default::default(),
            default_ring_pattern: RingPattern::try_parse(config.default_ring_pattern.as_str()).unwrap_or_else(|| {
                warn!("Unable to read default ring pattern from config. Using fallback pattern.");
                RingPattern {
                    components: vec![RingPatternComponent::Ring(Duration::from_secs(2)), RingPatternComponent::Low(Duration::from_secs(4))]
                }
            })
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
                match (cbuf[0] as char).to_ascii_lowercase() {
                    'i' => tx.send(PhoneInputSignal::HookState(false)).unwrap(),
                    'o' => tx.send(PhoneInputSignal::HookState(true)).unwrap(),
                    'w' => {                        
                        tx.send(PhoneInputSignal::RotaryDialRest(false)).unwrap();
                        thread::sleep(time::Duration::from_millis(350));
                    },
                    'e' => {
                        tx.send(PhoneInputSignal::RotaryDialPulse).unwrap();
                        thread::sleep(time::Duration::from_millis(80));
                    }
                    'r' => tx.send(PhoneInputSignal::RotaryDialRest(true)).unwrap(),
                    digit @ '0'..='9' | digit @ 'a'..='d' | digit @ '*' | digit @ '#' => {
                        thread::sleep(time::Duration::from_millis(200));
                        tx.send(PhoneInputSignal::Digit(digit.to_ascii_uppercase())).unwrap();
                    },
                    'f' => tx.send(PhoneInputSignal::Coin(1)).unwrap(),
                    'g' => tx.send(PhoneInputSignal::Coin(5)).unwrap(),
                    'h' => tx.send(PhoneInputSignal::Coin(10)).unwrap(),
                    'j' => tx.send(PhoneInputSignal::Coin(25)).unwrap(),
                    '-' => thread::sleep(time::Duration::from_millis(250)),
                    '_' => thread::sleep(time::Duration::from_millis(500)),
                    '.' => thread::sleep(time::Duration::from_millis(1000)),
                    _ => {}
                };
            }
        });
        (thread, rx)
    }
}

impl PhoneEngine {
    pub fn tick(&self) {
        // Process GPIO inputs
        if let Ok(signal) = self.rx_gpio.try_recv() {
            use PhoneInputSignal::*;

            // Perform any additional processing here before passing on the signal
            #[allow(unused_variables)]
            match signal {
                Digit(digit) => {
                    self.sound_engine.borrow().play_dtmf(digit, self.dtmf_tone_duration, 1.0);
                },
                _ => {}
            }

            self.send_to_engine(signal);
        }

        // Process GPIO outputs
        if let Some(rx_engine) = self.rx_engine.borrow().as_ref() {
            if let Ok(signal) = rx_engine.try_recv() {
                use PhoneOutputSignal::*;
                match signal {
                    Ring(pattern) => {
                        trace!("Ringer: {:?}", pattern);
                        
                        // Send ring status to GPIO
                        if let Some(tx_ringer) = &self.tx_ringer {
                            tx_ringer.send(pattern.clone()).expect("Ringer TX channel is dead");
                        }

                        // Play sound on PC
                        #[cfg(not(feature = "rpi"))]
                        {
                            if pattern.is_some() {
                                self.sound_engine.borrow().play("rings/ring_spkr_*", Channel::SignalOut, 
                                false, 
                                true, 
                                SoundPlayOptions {
                                    looping: true,
                                    .. Default::default()
                                });
                            } else {
                                self.sound_engine.borrow().stop(Channel::SignalOut)
                            }
                        }
                    },
                }
            }
        }
    }

    fn send_to_engine(&self, input: PhoneInputSignal) -> bool {
        if let Some(tx) = self.tx_engine.borrow().as_ref() {
            tx.send(input).unwrap();
            return true;
        }
        false
    }

    /// Creates a messaging channel for the PBX to listen to input signals from the phone.
    pub fn gen_phone_output(&self) -> mpsc::Receiver<PhoneInputSignal> {
        let (tx_pbx, rx_input) = mpsc::channel();
        self.tx_engine.replace(Some(Rc::new(tx_pbx)));
        rx_input
    }

    pub fn listen(&self, input_from_pbx: mpsc::Receiver<PhoneOutputSignal>) {
        self.rx_engine.replace(Some(Rc::new(input_from_pbx)));
    }
}