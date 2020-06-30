#![allow(dead_code)]
#![allow(unreachable_patterns)]

mod props;
mod api;
mod sm;

use std::rc::Rc;
use std::cell::RefCell;
use std::{thread, time, fs, sync::mpsc};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::time::{Instant, Duration};
use rand::Rng;
use mlua::prelude::*;
use indexmap::IndexMap;
use log::{info, warn, trace, error};
use crate::sound::*;
use crate::phone::*;
use crate::config::*;

pub use self::api::*;
pub use self::props::*;
pub use self::sm::*;

/// `Option<Rc<T>>`
type Orc<T> = Option<Rc<T>>;

/// `Rc<RefCell<T>>`
type RcRefCell<T> = Rc<RefCell<T>>;

// Script path constants
const BOOTSTRAPPER_SCRIPT_NAME: &str = "bootstrapper";
const API_GLOB: &str = "api/*";

// Pulse dialing digits
const PULSE_DIAL_DIGITS: &[u8] = b"1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ";

const DEFAULT_FIRST_PULSE_DELAY_MS: u64 = 200;

type ServiceId = usize;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PbxState {
    Idle,
    IdleRinging,
    DialTone,
    PDD,
    CallingOut,
    Connected,
    Busy
}

/// A Lua-powered telephone exchange that loads,
/// manages, and runs scripted phone services.
pub struct PbxEngine<'lua> {
    /// Type of the host phone.
    host_phone_type: PhoneType,
    /// The Lua context associated with the engine.
    lua: Lua,
    /// The root directory from which Lua scripts are loaded.
    scripts_root: PathBuf,
    /// The starting time of the engine.
    start_time: Instant,
    /// The numbered services associated with the engine.
    phone_book: RefCell<HashMap<String, ServiceId>>,
    /// The services (both numbered and otherwise) associated with the engine.
    services: RefCell<IndexMap<String, Rc<ServiceModule<'lua>>>>,
    /// The sound engine associated with the engine.
    sound_engine: RcRefCell<SoundEngine>,
    /// The intercept service.
    intercept_service: RefCell<Orc<ServiceModule<'lua>>>,
    /// Channel for sending output signals to the host phone.
    phone_output: RefCell<Option<mpsc::Sender<PhoneOutputSignal>>>,
    /// Channel for receiving input signals from the host phone.
    phone_input: RefCell<Option<mpsc::Receiver<PhoneInputSignal>>>,
    /// The service to which the PBX is connecting/has connected the host.
    other_party: RefCell<Orc<ServiceModule<'lua>>>,
    /// The current state of the engine.
    state: RefCell<PbxState>,
    /// Time when PDD last started.
    pdd_start: RefCell<Instant>,
    /// Time when the current state started.
    state_start: RefCell<Instant>,
    /// Phone configuration.
    config: Rc<CursedConfig>,
    /// Post-dial delay.
    post_dial_delay: Duration,
    /// The currently dialed number.
    dialed_number: RefCell<String>,
    /// Off-hook delay.
    off_hook_delay: Duration,
    /// Enable switchhook dialing?
    switch_hook_dialing_enabled: bool,
    /// Amount of money (given in lowest denomination, e.g. cents) that is credited for the next call.
    /// > _"Kajhiit has calls, if you have coin."_
    coin_deposit: RefCell<u32>,
    /// Indicates whether the initial coin deposit for the call has been consumed
    initial_deposit_consumed: RefCell<bool>,
    /// Delay before coins get eaten after call is accepted
    coin_consume_delay: Duration,
    /// Allows services to set their own prices.
    enable_custom_service_rates: bool,
    /// The default rate applied to calls.
    standard_call_rate: u32,
    /// Last known state of the host's hookswitch.
    host_on_hook: RefCell<bool>,
    /// Time of the last staet change of the host's hookswitch.
    host_hook_change_time: RefCell<Instant>,
    /// Is host rotary dial resting?
    host_rotary_resting: RefCell<bool>,
    /// Number of host pulses since last dialed digit.
    host_rotary_pulses: RefCell<usize>,
    /// Time of the last lifting of the rotary dial rest switch.
    host_rotary_dial_lift_time: RefCell<Instant>,
    /// Delay between rotary dial leaving resting state and first valid pulse.
    host_rotary_first_pulse_delay: Duration,
}

#[allow(unused_must_use)]
impl<'lua> PbxEngine<'lua> {
    pub fn new(scripts_root: impl Into<String>, config: &Rc<CursedConfig>, sound_engine: &Rc<RefCell<SoundEngine>>) -> Self {
        let lua = Lua::new();
        let now = Instant::now();
        let host_phone_type = PhoneType::from_name(config.phone_type.as_str());
        let (coin_consume_delay_ms, standard_call_rate, enable_custom_service_rates) 
        = if let Some(ppcfg) = config.payphone.as_ref() {
            (
                ppcfg.coin_consume_delay_ms.unwrap_or(0),
                ppcfg.standard_call_rate.unwrap_or(0),
                ppcfg.enable_custom_service_rates.unwrap_or(true)
            )
        } else {
            (0, 0, true)
        };
        Self {
            host_phone_type,
            lua,
            start_time: now,
            pdd_start: RefCell::new(now),
            scripts_root: Path::new(scripts_root.into().as_str()).canonicalize().unwrap(),
            config: Rc::clone(config),
            sound_engine: Rc::clone(sound_engine),
            phone_book: Default::default(),
            services: Default::default(),
            intercept_service: Default::default(),
            state: RefCell::new(PbxState::Idle),
            state_start: RefCell::new(now),
            phone_input: Default::default(),
            phone_output: Default::default(),
            off_hook_delay: Duration::from_secs_f32(config.off_hook_delay),
            post_dial_delay: Duration::from_secs_f32(config.pdd),
            other_party: Default::default(),
            dialed_number: Default::default(),
            switch_hook_dialing_enabled: config.features.enable_switch_hook_dialing.unwrap_or(false),
            coin_deposit: RefCell::new(0),
            coin_consume_delay: Duration::from_millis(coin_consume_delay_ms),
            enable_custom_service_rates,
            standard_call_rate,
            initial_deposit_consumed: RefCell::new(false),
            host_hook_change_time: RefCell::new(now),
            host_on_hook: RefCell::new(true),
            host_rotary_pulses: Default::default(),
            host_rotary_resting: RefCell::new(true),
            host_rotary_dial_lift_time: RefCell::new(now),
            host_rotary_first_pulse_delay: Duration::from_millis(config.rotary_first_pulse_delay_ms.unwrap_or(DEFAULT_FIRST_PULSE_DELAY_MS)),
        }
    }

    pub fn gen_pbx_output(&self) -> mpsc::Receiver<PhoneOutputSignal> {
        let (tx, rx) = mpsc::channel();
        self.phone_output.replace(Some(tx));
        rx
    }

    pub fn listen_phone_input(&self, input_from_phone: mpsc::Receiver<PhoneInputSignal>) {
        self.phone_input.replace(Some(input_from_phone));
    }

    fn send_output(&self, signal: PhoneOutputSignal) -> bool {
        if let Some(tx) = self.phone_output.borrow().as_ref() {
            tx.send(signal).is_ok();
        }
        false
    }

    fn lookup_service_id(&self, id: ServiceId) -> Orc<ServiceModule> {
        self.services.borrow().get_index(id).map(|result| Rc::clone(result.1))
    }

    /// Searches the phone directory for the specified number and returns the service associated with it, or `None` if the number is unassigned.
    fn lookup_service(&self, phone_number: &str) -> Orc<ServiceModule> {
        if let Some(id) = self.phone_book.borrow().get(phone_number) {
            return self.lookup_service_id(*id);
        }
        None
    }

    /// Calls the specified phone number.
    fn call_number(&'lua self, number: &str) -> bool {
        info!("Placing call to: {}", number);
        if let Some(service) = self.lookup_service(number) {
            self.call_service(service);
            return true;
        } else {
            self.call_intercept(CallReason::NumberDisconnected);
            return false;
        }
    }

    /// Calls the specified service.
    fn call_service(&'lua self, service: Rc<ServiceModule>) {
        use PbxState::*;
        match self.state() {
            DialTone | Busy | PDD => {
                info!("PBX: Connecting call -> {} ({:?})", service.name(), service.phone_number());
                // Inform the service state machine that the user initiated the call
                service.set_reason(CallReason::UserInit);
                // Set other_party to requested service
                let service = self.lookup_service_id(service.id().unwrap()).unwrap();
                self.load_other_party(Rc::clone(&service));
                // Set PBX to call-out state
                self.set_state(CallingOut);
            },
            _ => {}
        }
    }

    /// Calls the intercept service, if available.
    fn call_intercept(&'lua self, reason: CallReason) {
        if let Some(intercept_service) = self.intercept_service.borrow().as_ref() {
            self.call_service(Rc::clone(intercept_service));
            intercept_service.set_reason(reason);
        } else {
            // Default to busy signal if there is no intercept service
            warn!("PBX: No intercept service; defaulting to busy signal.");
            self.set_state(PbxState::Busy);
        }
    }

    #[inline]
    pub fn state(&self) -> PbxState {
        self.state.borrow().clone()
    }

    #[inline]
    fn update_pdd_start(&self) {
        self.pdd_start.replace(Instant::now());
    }

    #[inline]
    fn pdd_time(&self) -> Duration {
        if self.state() == PbxState::PDD {
            return Instant::now().saturating_duration_since(*self.pdd_start.borrow());
        }
        Duration::default()
    }

    #[inline]
    fn clear_dialed_number(&self) {
        self.dialed_number.borrow_mut().clear();
    }

    #[inline]
    fn consume_dialed_digit(&self) -> Option<char> {
        self.dialed_number.borrow_mut().pop()
    }

    fn get_dialed_number(&self) -> String {
        let dialed: String = self.dialed_number.borrow().clone();
        dialed
    }

    fn run_script(&self, name: &str) -> Result<(), String> {
        let path = self.resolve_script_path(name);
        match fs::read_to_string(&path) {
            Ok(lua_src) => self.lua.load(&lua_src).set_name(name).unwrap().exec(),
            Err(err) => return Err(format!("Failed to run lua file '{}': {:#?}", path.to_str().unwrap(), err))
        };

        Ok(())
    }

    fn run_scripts_in_glob(&self, glob: &str) -> Result<(), String> {
        let search_path = self.resolve_script_path(glob);
        let search_path_str = search_path.to_str().expect("Failed to create search pattern from glob");
        for entry in globwalk::glob(search_path_str).expect("Unable to read script search pattern") {
            if let Ok(dir) = entry {
                let script_path = dir.path().canonicalize().expect("Unable to expand service module path");
                let script_path_str = script_path.to_str().unwrap();
                info!("PBX: Loading Lua API: {:?}", script_path.file_name());
                match fs::read_to_string(&script_path) {
                    Ok(lua_src) => self.lua.load(&lua_src).set_name(script_path_str).unwrap().exec(),
                    Err(err) => return Err(format!("Failed to run lua file '{}': {:#?}", script_path_str, err))
                };
            }
        }
        Ok(())
    }

    fn resolve_script_path(&self, name: &str) -> PathBuf {
        self.scripts_root.join(name).with_extension("lua")
    }

    pub fn load_services(&'lua self) {
        self.phone_book.borrow_mut().clear();
        let search_path = self.scripts_root.join("services").join("**").join("*.lua");
        let search_path_str = search_path.to_str().expect("Failed to create search pattern for service modules");
        let mut services = self.services.borrow_mut();
        let mut services_numbered = self.phone_book.borrow_mut();
        for entry in globwalk::glob(search_path_str).expect("Unable to read search pattern for service modules") {
            if let Ok(dir) = entry {
                let module_path = dir.path().canonicalize().expect("Unable to expand service module path");
                let service = ServiceModule::from_file(&self.lua, &module_path);
                match service {
                    Ok(service) => {
                        // Register service
                        let service_name = service.name().to_owned();
                        let service_role = service.role();
                        let service_phone_number = service.phone_number().clone();
                        let service = Rc::new(service);
                        let (service_id, _) = services.insert_full(service_name, Rc::clone(&service));
                        service.register_id(service_id);

                        // Register service number
                        if let Some(phone_number) = service_phone_number {
                            if !phone_number.is_empty() {
                                services_numbered.insert(phone_number, service_id);
                            }
                        }

                        // Register intercept service
                        match service_role {
                            ServiceRole::Intercept => {
                                self.intercept_service.replace(Some(Rc::clone(&service)));
                            },
                            _ => {}
                        }

                        info!("Service loaded: {} (N = {:?}, ID = {:?})", service.name(), service.phone_number(), service.id());
                    },
                    Err(err) => {
                        error!("Failed to load service module '{:?}': {:#?}", module_path, err);
                    }
                }
            }
        }
    }

    /// Gets the other party associated with the active or pending call.
    fn get_other_party_service(&self) -> Orc<ServiceModule> {
        if let Some(service) = self.other_party.borrow().as_ref() {
            return Some(Rc::clone(service));
        }
        None
    }

    /// Removes the other party and unloads any associated non-static resources.
    fn unload_other_party(&self) {
        if let Some(service) = self.other_party.borrow().as_ref() {
            service.transition_state(ServiceState::Idle);
            service.unload_sound_banks(&self.sound_engine);
        }
        self.other_party.replace(None);
    }

    /// Sets the other party to the specified service.
    fn load_other_party(&self, service: Rc<ServiceModule<'lua>>) {
        service.load_sound_banks(&self.sound_engine);
        if let Some(prev_service) = self.other_party.replace(Some(service)) {
            prev_service.unload_sound_banks(&self.sound_engine);
        }
    }

    fn play_comfort_noise(&self) {
        let sound_engine = self.sound_engine.borrow();
        if sound_engine.channel_busy(Channel::NoiseIn) { return }
        sound_engine.play(
            self.config.sound.comfort_noise_name.as_str(), 
            Channel::NoiseIn,
            false,
            true,
            true,
            1.0,
            self.config.sound.comfort_noise_volume
        );
    }

    /// Sets the current state of the engine.
    fn set_state(&'lua self, state: PbxState) {
        use PbxState::*;
        if *self.state.borrow() == state {
            return;
        }

        let prev_state = self.state.replace(state);
        let state_start = Instant::now();
        let last_state_start = self.state_start.replace(state_start);
        let state_time = state_start.saturating_duration_since(last_state_start);

        // Run behavior for state we're leaving
        match prev_state {
            PbxState::IdleRinging => {
                self.send_output(PhoneOutputSignal::Ring(false));
            },
            _ => {}
        }

        // Run behavior for new state
        match state {
            Idle | IdleRinging => {},
            _ => {
                self.play_comfort_noise();
            }
        }

        // Run behavior for state transition
        match (prev_state, state) {
            // TODO: Implement all valid PBX state transitions
            (_, Idle) => {
                self.unload_other_party();
                self.sound_engine.borrow().stop_all_except(Channel::SignalOut);
                self.clear_dialed_number();
            },
            (_, IdleRinging) => {
                self.send_output(PhoneOutputSignal::Ring(true));
            },
            (_, DialTone) => {
                self.sound_engine.borrow().play_dial_tone();
            },
            (_, Busy) => {
                self.unload_other_party();
                let sound_engine = self.sound_engine.borrow();
                sound_engine.stop_all_nonsignal();
                sound_engine.play_busy_tone();
            },
            (_, PDD) => {
                // Stop any PBX signals
                self.sound_engine.borrow().stop(Channel::SignalIn);
                self.update_pdd_start();
            },
            (_, CallingOut) => {
                if let Some(service) = self.get_other_party_service() {
                    self.clear_dialed_number();
                    self.sound_engine.borrow().stop(Channel::SignalIn);

                    // Tell service that we're calling it
                    service.transition_state(ServiceState::IncomingCall);

                    // Finally, play the ringback tone (if we're allowed to)
                    if service.ringback_enabled() {
                        self.sound_engine.borrow().play_ringback_tone();
                    }
                } else {
                    warn!("No remote party specified when calling out.");
                    self.call_intercept(CallReason::NumberDisconnected);
                }
            },
            (_, Connected) => {
                self.initial_deposit_consumed.replace(false);
                self.clear_dialed_number();
                // Stop all existing sounds except for host signals
                let sound_engine = self.sound_engine.borrow();
                sound_engine.stop(Channel::SignalIn);
                // Transition connecting service to call state
                if let Some(service) = self.other_party.borrow().as_ref() {
                    service.transition_state(ServiceState::Call);
                }
            }
            _ => {}
        }

        info!("PBX: {:?} ({:?}) --> {:?}", prev_state, state_time, state);
    }

    /// Called when an off-hook timeout occurs.
    #[inline]
    fn handle_off_hook_timeout(&'lua self) {
        info!("PBX: Off-hook timeout.");
        self.call_intercept(CallReason::OffHook);
    }

    /// Called when the host dials a digit via any method.
    #[inline]
    fn handle_host_digit(&'lua self, digit: char) {
        use PbxState::*;

        // Perform special digit-triggered behaviors
        match self.state() {
            // Ignore digits dialed while the phone is on the hook
            Idle | IdleRinging => return,

            // Transition from dial tone to PDD once the first digit is dialed
            DialTone => {
                self.set_state(PbxState::PDD);
            },

            // Reset the PDD timer each time a digit is dialed
            PDD => {
                self.update_pdd_start();
            },
            _ => {}
        }

        info!("PBX: Digit '{}'", digit);

        // Add digit to dialed number
        self.dialed_number.borrow_mut().push(digit);
    }

    /// Called when the engine receives a pulse from the host's rotary dial.
    #[inline]
    fn handle_rotary_pulse(&self) {
        match self.state() {
            PbxState::Idle | PbxState::IdleRinging => return,
            _ => {
                let current_rest_state = *self.host_rotary_resting.borrow();
                if !current_rest_state {
                    // This is a fix for my noisy rotary dial randomly pulsing when I lift it from resting.
                    // Forcing a delay between the dial lift and the first pulse seems to resolve this issue.
                    let rotary_rest_lifted_time = self.host_rotary_dial_lift_time.borrow().elapsed();
                    if rotary_rest_lifted_time > self.host_rotary_first_pulse_delay {
                        // Increment pulse count
                        self.host_rotary_pulses.replace_with(|&mut old| old + 1);
                        self.sound_engine.borrow().play("rotary/pulse", Channel::SignalOut, false, false, true, 1.0, 1.0);
                    } else {
                        trace!("PBX: Discarded premature rotary dial pulse");
                    }
                }
            }
        }
    }

    fn add_coin_deposit(&self, cents: u32) {
        let mut total = 0;
        self.coin_deposit.replace_with(|prev_cents| { total = *prev_cents + cents; total });
        info!("PBX: Deposited {}¢ (total: {}¢)", cents, total);
    }

    fn consume_coin_deposit(&self) {
        self.coin_deposit.replace(0);
        self.initial_deposit_consumed.replace(true);
        info!("PBX: Coin deposit cleared.");
    }

    fn initial_deposit_consumed(&self) -> bool {
        *self.initial_deposit_consumed.borrow()
    }

    /// Called when the user deposits a coin.
    #[inline]
    fn handle_coin_deposit(&self, cents: u32) {
        self.add_coin_deposit(cents);
    }

    /// Called when the resting state of the host's rotary dial changes.
    #[inline]
    fn handle_rotary_rest_state(&'lua self, resting: bool) {
        if resting == self.host_rotary_resting.replace(resting) {return}

        if resting {
            // Ignore 
            match self.state() {
                PbxState::Idle | PbxState::IdleRinging => {},
                _ => {
                    // When dial moves to resting, dial digit according to pulse count
                    let digit_num = *self.host_rotary_pulses.borrow();
                    if digit_num < PULSE_DIAL_DIGITS.len() && digit_num > 0 {
                        let digit = PULSE_DIAL_DIGITS[digit_num - 1] as char;
                        self.handle_host_digit(digit);
                    }
                }
            }
            
        } else {
            // When dial moves away from resting, reset pulse count
            self.host_rotary_pulses.replace(0);
            self.host_rotary_dial_lift_time.replace(Instant::now());
        }
    }

    #[inline]
    fn handle_hook_state_change(&'lua self, on_hook: bool) {
        use PbxState::*;
        let state = self.state();
        let hook_change_time = Instant::now();
        if on_hook {
            match state {
                Idle | IdleRinging => {}
                _ => {
                    info!("PBX: Host on-hook.");
                    self.set_state(PbxState::Idle);
                }
            }
        } else {
            // Only process this signal if the line is inactive or ringing
            match state {
                // Picking up idle phone
                Idle => {
                    info!("PBX: Host off-hook.");
                    self.set_state(PbxState::DialTone);
                },
                // Answering a call
                IdleRinging => {
                    info!("PBX: Host off-hook, connecting call.");
                    // Connect the call
                    self.set_state(PbxState::Connected);
                },
                _ => {}
            }
        }
    }

    /// Reads and handles pending input signals from the host device.
    fn process_input_signals(&'lua self) {
        if let Some(phone_input) = self.phone_input.borrow().as_ref() {
            while let Ok(signal) = phone_input.try_recv() {
                use PhoneInputSignal::*;
                match signal {
                    HookState(on_hook) => self.handle_hook_state_change(on_hook),
                    RotaryDialRest(resting) => self.handle_rotary_rest_state(resting),
                    RotaryDialPulse => self.handle_rotary_pulse(),
                    Motion => {
                        info!("PBX: Detected motion.");
                        // TODO: Process motion sensor inputs
                    },
                    Digit(digit) => {
                        self.handle_host_digit(digit);
                    },
                    Coin(cents) => {
                        self.handle_coin_deposit(cents);
                    }
                }
            }
        }
    }

    /// Gets the length of time for which the current state has been active.
    #[inline]
    pub fn current_state_time(&self) -> Duration {
        Instant::now().saturating_duration_since(*self.state_start.borrow())
    }

    /// Updates the state of the engine.
    #[inline]
    fn update_pbx_state(&'lua self) {
        use PbxState::*;
        let state = self.state();
        match state {
            DialTone => {
                let state_time = self.current_state_time();
                if state_time.as_secs_f32() >= self.config.off_hook_delay {
                    self.handle_off_hook_timeout();
                }
            }
            PDD => {
                if self.pdd_time() >= self.post_dial_delay {
                    match self.host_phone_type {
                        PhoneType::Payphone => {
                            let number_to_dial = self.get_dialed_number();

                            // Figure out how much the call costs
                            let price = match self.lookup_service(number_to_dial.as_str()) {
                                Some(service_to_call) if self.enable_custom_service_rates => 
                                match service_to_call.custom_price() {
                                    Some(cents) => cents,
                                    None => self.standard_call_rate
                                },
                                _ => self.standard_call_rate
                            };

                            // If the user has deposited enough money, call the number. Otherwise, do nothing.
                            // TODO: Play a message if the user has not deposited enough coins.
                            if *self.coin_deposit.borrow() >= price {
                                self.call_number(number_to_dial.as_str());
                            }
                        },
                        _ => {
                            let number_to_dial = self.get_dialed_number();
                            self.call_number(number_to_dial.as_str());
                        }
                    }
                }
            },
            Connected => {
                // Wait for user-configured delay and eat coin deposit
                // TODO: Add support for "charge-per-minute" calls
                if !self.initial_deposit_consumed() && self.current_state_time() >= self.coin_consume_delay {
                    self.consume_coin_deposit();
                }
            }
            _ => {}
        }
    }

    /// Updates the states of the services associated with the engine.
    #[inline]
    fn update_services(&'lua self) {
        use ServiceIntent::*;
        use PbxState::*;
        let state = self.state();
        let service_modules = self.services.borrow();
        let service_iter = service_modules.iter();
        for (_, service) in service_iter {
            let mut intent = service.tick(ServiceData::None);
            loop {
                match intent {
                    // Service requests a digit from the user
                    Ok(ReadDigit) => {
                        if let Some(digit) = self.consume_dialed_digit() {
                            intent = service.tick(ServiceData::Digit(digit));
                            continue;
                        }
                    },
                    // Service wants to call the user
                    Ok(CallUser) => {
                        // First, check that there's nobody on the line and the user's on-hook.
                        // Also make sure that the config allows incoming calls.
                        if self.config.features.enable_incoming_calls.unwrap_or(false) 
                        && self.state() == PbxState::Idle 
                        && self.other_party.borrow().is_none() {
                            service.set_reason(CallReason::ServiceInit);
                            service.transition_state(ServiceState::OutgoingCall);
                            self.load_other_party(Rc::clone(service));
                            self.set_state(PbxState::IdleRinging);
                        } else {
                            // Tell the service they're busy
                            intent = service.tick(ServiceData::LineBusy);
                            continue;
                        }
                    },
                    // Service wants to accept incoming call
                    Ok(AcceptCall) => {
                        if state == CallingOut { 
                            self.set_state(Connected);
                        }
                    },
                    // Service wants to end current call
                    Ok(EndCall) => {
                        match state {
                            // Transition to idle (hangs up at end of CALL state)
                            Connected => {
                                service.transition_state(ServiceState::Idle);
                            },
                            // Caller has given up, disconnect immediately
                            IdleRinging => {
                                service.transition_state(ServiceState::Idle);
                                self.set_state(PbxState::Idle);
                            },
                            _ => {}
                        }
                    }
                    // Service has just exited CALL state
                    Ok(StateEnded(ServiceState::Call)) => {
                        // Don't affect PBX state if the call is already ended
                        match state {
                            Connected => {
                                // TODO: Allow user to customize behavior when service ends call
                                self.set_state(Busy);
                            },
                            _ => {}
                        }
                    },
                    Ok(_) => (),
                    Err(err) => {
                        self.sound_engine.borrow().play_panic_tone();
                        match err {
                            LuaError::RuntimeError(msg) => error!("LUA ERROR: {}", msg),
                            _ => error!("LUA ERROR: {:?}", err)
                        }
                    }
                }
                break;
            }
        }
    }

    /// Processes pending inputs and updates state information associated with the engine.
    pub fn tick(&'lua self) {
        self.process_input_signals();
        self.update_pbx_state();
        self.update_services();
    }
}