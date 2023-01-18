#![allow(dead_code)]
#![allow(unreachable_patterns)]

mod props;
mod api;
mod agent;

use std::rc::Rc;
use std::cell::{RefCell, Cell};
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
pub use self::agent::*;

#[cfg(feature = "rpi")]
use crate::gpio::*;

/// `Option<Rc<T>>`
type Orc<T> = Option<Rc<T>>;

/// `Rc<RefCell<T>>`
type RcRefCell<T> = Rc<RefCell<T>>;

// Script path constants
const SETUP_SCRIPT_NAME: &str = "setup";
const AGENTS_PATH_NAME: &str = "agents";
const API_GLOB: &str = "api/*";

// Pulse dialing digits
const PULSE_DIAL_DIGITS: &[u8] = b"1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ";

// Vertical Service Codes
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VerticalServiceCode {
    /// Last-Call Return (Tone: *69, Pulse: 1169)
    LastCallReturn
}

impl VerticalServiceCode {
    #[inline(always)]
    pub fn parse(number: &str, phone_type: PhoneType) -> Option<Self> {
        Some(match (number, phone_type) {
            ("1169", t @ PhoneType::Rotary) | ("*69", t @ _) if t != PhoneType::TouchTone => Self::LastCallReturn,
            _ => return None
        })
    }
}

const DEFAULT_FIRST_PULSE_DELAY_MS: u64 = 200;

type AgentId = usize;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PhoneLineState {
    /// The phone is on-hook and the line is idle.
    Idle,
    /// The phone is on-hook and ringing. An agent is calling the phone.
    IdleRinging,
    /// The phone is off-hook and the line is awaiting a number and transmitting a dial tone.
    DialTone,
    /// The phone has dialed something and the line is in Post-Dial Delay.
    PDD,
    /// The line is calling out to an agent and awaiting connection. The line may transmit a ringback tone.
    CallingOut,
    /// The line is connected to an agent and a call is in-progress.
    Connected,
    /// The phone is off-hook, no call is connected, and the line is transmitting a busy signal.
    Busy
}

/// A Lua-powered telephone exchange that loads,
/// manages, and runs scripted agents.
pub struct CursedEngine<'lua> {
    /// Type of the host phone.
    host_phone_type: PhoneType,
    /// The Lua context associated with the engine.
    lua: Lua,
    /// The root directory from which Lua scripts are loaded.
    scripts_root: PathBuf,
    /// The starting time of the engine.
    start_time: Instant,
    /// The numbered agents associated with the engine.
    phone_book: RefCell<HashMap<String, AgentId>>,
    /// The last agent who called the host
    last_caller_id: Cell<Option<AgentId>>,
    /// The agents (both numbered and otherwise) associated with the engine.
    agents: RefCell<IndexMap<String, Rc<AgentModule<'lua>>>>,
    /// The sound engine associated with the engine.
    sound_engine: RcRefCell<SoundEngine>,
    /// The intercept agent.
    intercept_agent: RefCell<Orc<AgentModule<'lua>>>,
    /// Channel for sending output signals to the host phone.
    phone_output: RefCell<Option<mpsc::Sender<PhoneOutputSignal>>>,
    /// Channel for receiving input signals from the host phone.
    phone_input: RefCell<Option<mpsc::Receiver<PhoneInputSignal>>>,
    /// The agent to which the engine is connecting/has connected the host.
    other_party: RefCell<Orc<AgentModule<'lua>>>,
    /// The current state of the engine.
    state: RefCell<PhoneLineState>,
    /// Time when PDD last started.
    pdd_start: RefCell<Instant>,
    /// Time when the current state started.
    state_start: RefCell<Instant>,
    /// Phone configuration.
    config: Rc<CursedConfig>,
    /// The currently dialed number.
    dialed_number: RefCell<String>,
    /// Enable switchhook dialing?
    switch_hook_dialing_enabled: bool,
    /// Amount of money (given in lowest denomination, e.g. cents) that is credited for the next call.
    coin_deposit: RefCell<u32>,
    /// Indicates whether the initial coin deposit for the call has been consumed.
    initial_deposit_consumed: RefCell<bool>,
    /// Indicates whether the PBX is waiting for the initial deposit to start the call.
    awaiting_initial_deposit: RefCell<bool>,
    /// Amount of time credited for the current (or next) call.
    time_credit: RefCell<Duration>,
    /// Delay before coins get eaten after call is accepted
    coin_consume_delay: Duration,
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
    /// GPIO interface used by Lua.
    #[cfg(feature = "rpi")]
    gpio: crate::gpio::GpioInterface,
}

#[allow(unused_must_use)]
impl<'lua> CursedEngine<'lua> {
    pub fn new(scripts_root: impl Into<String>, config: &Rc<CursedConfig>, sound_engine: &Rc<RefCell<SoundEngine>>) -> Self {
        let lua = Lua::new();
        let now = Instant::now();
        let host_phone_type = PhoneType::from_name(config.phone_type.as_str());
        
        Self {
            host_phone_type,
            lua,
            start_time: now,
            pdd_start: RefCell::new(now),
            scripts_root: Path::new(scripts_root.into().as_str()).canonicalize().unwrap(),
            config: Rc::clone(config),
            sound_engine: Rc::clone(sound_engine),
            phone_book: Default::default(),
            last_caller_id: Cell::new(None),
            agents: Default::default(),
            intercept_agent: Default::default(),
            state: RefCell::new(PhoneLineState::Idle),
            state_start: RefCell::new(now),
            phone_input: Default::default(),
            phone_output: Default::default(),
            other_party: Default::default(),
            dialed_number: Default::default(),
            switch_hook_dialing_enabled: config.features.enable_switch_hook_dialing.unwrap_or(false),
            coin_deposit: RefCell::new(0),
            coin_consume_delay: Duration::from_millis(config.payphone.coin_consume_delay_ms),
            initial_deposit_consumed: RefCell::new(false),
            awaiting_initial_deposit: RefCell::new(false),
            time_credit: Default::default(),
            host_hook_change_time: RefCell::new(now),
            host_on_hook: RefCell::new(true),
            host_rotary_pulses: Default::default(),
            host_rotary_resting: RefCell::new(true),
            host_rotary_dial_lift_time: RefCell::new(now),
            host_rotary_first_pulse_delay: Duration::from_millis(config.rotary_first_pulse_delay_ms.unwrap_or(DEFAULT_FIRST_PULSE_DELAY_MS)),
            #[cfg(feature = "rpi")]
            gpio: GpioInterface::new().expect("Unable to initialize Lua GPIO interface"),
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

    fn lookup_agent_id(&self, id: AgentId) -> Orc<AgentModule> {
        self.agents.borrow().get_index(id).map(|result| Rc::clone(result.1))
    }

    /// Searches the phone directory for the specified number and returns the agent associated with it, or `None` if the number is unassigned.
    fn lookup_agent(&self, phone_number: &str) -> Orc<AgentModule> {
        if let Some(id) = self.phone_book.borrow().get(phone_number) {
            return self.lookup_agent_id(*id);
        }
        None
    }

    /// Calls the specified phone number.
    fn call_number(&'lua self, number: &str) -> bool {

        let vsc_agent = if let Some(vsc) = VerticalServiceCode::parse(number, self.host_phone_type) {
            match vsc {
                VerticalServiceCode::LastCallReturn => 
                    self.config.features.enable_lcr.unwrap_or_default()
                    .then(|| self.last_caller_id.get().and_then(|id| self.lookup_agent_id(id)))
                    .flatten()
            }
        } else {
            None
        };

        info!("Placing call to: {}", number);
        if let Some(agent) = vsc_agent.or_else(|| self.lookup_agent(number)) {
            self.call_agent(agent);
            return true;
        } else {
            self.call_intercept(CallReason::NumberDisconnected);
            return false;
        }
    }

    /// Calls the specified agent.
    fn call_agent(&'lua self, agent: Rc<AgentModule>) {
        use PhoneLineState::*;
        match self.state() {
            DialTone | Busy | PDD => {
                info!("PBX: Connecting call -> {} ({:?})", agent.name(), agent.phone_number());
                // Inform the agent state machine that the user initiated the call
                agent.set_reason(CallReason::UserInit);
                // Set other_party to requested agent
                let agent = self.lookup_agent_id(agent.id().unwrap()).unwrap();
                self.load_other_party(Rc::clone(&agent));
                // Set PBX to call-out state
                self.set_state(CallingOut);
            },
            _ => {}
        }
    }

    /// Calls the intercept agent, if available.
    fn call_intercept(&'lua self, reason: CallReason) {
        if let Some(intercept_agent) = self.intercept_agent.borrow().as_ref() {
            self.call_agent(Rc::clone(intercept_agent));
            intercept_agent.set_reason(reason);
        } else {
            // Default to busy signal if there is no intercept agent
            warn!("PBX: No intercept agent; defaulting to busy signal.");
            self.set_state(PhoneLineState::Busy);
        }
    }

    #[inline]
    pub fn state(&self) -> PhoneLineState {
        self.state.borrow().clone()
    }

    #[inline]
    fn update_pdd_start(&self) {
        self.pdd_start.replace(Instant::now());
    }

    #[inline]
    fn pdd_time(&self) -> Duration {
        if self.state() == PhoneLineState::PDD {
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
                let script_path = dir.path().canonicalize().expect("Unable to expand script path");
                let script_path_str = script_path.to_str().unwrap();
                info!("PBX: Loading API script: {:?}", script_path.file_name().unwrap());
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

    pub fn load_agents(&'lua self) {
        self.phone_book.borrow_mut().clear();
        let search_path = self.scripts_root.join(AGENTS_PATH_NAME).join("**").join("*.lua");
        let search_path_str = search_path.to_str().expect("Failed to create search pattern for agent modules");
        let mut agents = self.agents.borrow_mut();
        let mut agents_numbered = self.phone_book.borrow_mut();
        for entry in globwalk::glob(search_path_str).expect("Unable to read search pattern for agent modules") {
            if let Ok(dir) = entry {
                let module_path = dir.path().canonicalize().expect("Unable to expand agent module path");
                let agent = AgentModule::from_file(&self.lua, &module_path);
                match agent {
                    Ok(agent) => {
                        // Register agent
                        let agent_name = agent.name().to_owned();
                        let agent_role = agent.role();

                        // Don't load Tollmasters if this isn't a payphone
                        if agent_role == AgentRole::Tollmaster && self.host_phone_type != PhoneType::Payphone {
                            continue
                        }

                        let agent_phone_number = agent.phone_number().clone();
                        let agent = Rc::new(agent);
                        let (agent_id, _) = agents.insert_full(agent_name, Rc::clone(&agent));
                        agent.register_id(agent_id);

                        // Register agent number
                        if let Some(phone_number) = agent_phone_number {
                            if !phone_number.is_empty() {
                                agents_numbered.insert(phone_number, agent_id);
                            }
                        }

                        // Register intercept agent
                        match agent_role {
                            AgentRole::Intercept => {
                                self.intercept_agent.replace(Some(Rc::clone(&agent)));
                            },
                            _ => {}
                        }

                        info!("Agent loaded: {} (num = {}, id = {:?})", agent.name(), agent.phone_number().as_deref().unwrap_or("[RESTRICTED]"), agent.id());
                    },
                    Err(err) => {
                        error!("Failed to load agent module '{:?}': {:#?}", module_path, err);
                    }
                }
            }
        }
    }

    /// Gets the other party associated with the active or pending call.
    fn get_other_party_agent(&self) -> Orc<AgentModule> {
        if let Some(agent) = self.other_party.borrow().as_ref() {
            return Some(Rc::clone(agent));
        }
        None
    }

    /// Removes the other party and unloads any associated non-static resources.
    fn unload_other_party(&self) {
        if let Some(agent) = self.other_party.borrow().as_ref() {
            agent.transition_state(AgentState::Idle);
            agent.unload_sound_banks(&self.sound_engine);
        }
        self.other_party.replace(None);
    }

    /// Sets the other party to the specified agent.
    fn load_other_party(&self, agent: Rc<AgentModule<'lua>>) {
        agent.load_sound_banks(&self.sound_engine);
        if let Some(prev_agent) = self.other_party.replace(Some(agent)) {
            prev_agent.unload_sound_banks(&self.sound_engine);
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

    #[inline(always)]
    fn is_payphone(&self) -> bool {
        self.host_phone_type == PhoneType::Payphone
    }

    /// Sets the current state of the engine.
    fn set_state(&'lua self, state: PhoneLineState) {
        use PhoneLineState::*;
        if *self.state.borrow() == state {
            return;
        }
        
        let prev_state = self.state.replace(state);
        let state_start = Instant::now();
        let last_state_start = self.state_start.replace(state_start);
        let state_time = state_start.saturating_duration_since(last_state_start);

        // Make sure that canceled unpaid call doesn't keep pinging Tollmaster
        self.awaiting_initial_deposit.replace(false);

        // Run behavior for state we're leaving
        match prev_state {
            PhoneLineState::IdleRinging => {
                self.send_output(PhoneOutputSignal::Ring(false));
            },
            PhoneLineState::Connected => {
                if self.is_payphone() {
                    // When leaving the connected state, clear existing time credit
                    self.initial_deposit_consumed.replace(false);
                    self.clear_time_credit();
                }
            }
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
                if let Some(agent) = self.get_other_party_agent() {
                    self.clear_dialed_number();
                    self.sound_engine.borrow().stop(Channel::SignalIn);

                    // Tell agent that we're calling it
                    agent.transition_state(AgentState::IncomingCall);

                    // Finally, play the ringback tone (if we're allowed to)
                    if agent.ringback_enabled() {
                        self.sound_engine.borrow().play_ringback_tone();
                    }
                } else {
                    warn!("No remote party specified when calling out.");
                    self.call_intercept(CallReason::NumberDisconnected);
                }
            },
            (_, Connected) => {
                self.clear_dialed_number();
                // Stop all existing sounds except for host signals
                let sound_engine = self.sound_engine.borrow();
                sound_engine.stop(Channel::SignalIn);
                // Transition connecting agent to call state
                if let Some(agent) = self.other_party.borrow().as_ref() {
                    agent.transition_state(AgentState::Call);
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
        use PhoneLineState::*;

        // Perform special digit-triggered behaviors
        match self.state() {
            // Ignore digits dialed while the phone is on the hook
            Idle | IdleRinging => return,

            // Transition from dial tone to PDD once the first digit is dialed
            DialTone => {
                self.set_state(PhoneLineState::PDD);
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
            PhoneLineState::Idle | PhoneLineState::IdleRinging => return,
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

    pub fn remaining_time_credit(&self) -> Duration {
        match self.state() {
            PhoneLineState::Connected => self.time_credit.borrow().checked_sub(self.current_state_time()).unwrap_or_default(),
            _ => *self.time_credit.borrow()
        }
    }

    fn clear_time_credit(&self) {
        self.time_credit.replace(Duration::default());
        info!("PBX: Time credit cleared.");
    }

    #[inline]
    fn has_time_credit(&self) -> bool {
        self.config.payphone.time_credit_seconds == 0 || self.remaining_time_credit().as_nanos() > 0
    }

    pub fn is_current_call_free(&self) -> bool {
        match self.host_phone_type {
            // If the payphone has a standard rate of 0 and custom rates are ignored, it's free
            PhoneType::Payphone if !self.config.payphone.enable_custom_agent_rates && self.config.payphone.standard_call_rate == 0
            => true,
            // Check rate of currently connected agent
            PhoneType::Payphone => {
                if let Some(agent) = self.get_other_party_agent().as_ref() {
                    match agent.role() {
                        AgentRole::Intercept => true,
                        _ => agent.custom_price().unwrap_or(self.config.payphone.standard_call_rate) == 0
                    }
                } else {
                    false
                }
            },
            _ => false
        }
    }

    pub fn current_call_rate(&self) -> u32 {
        self.get_other_party_agent()
            .as_ref()
            .map(|serv| serv.custom_price().filter(|_| self.config.payphone.enable_custom_agent_rates))
            .flatten()
            .unwrap_or(self.config.payphone.standard_call_rate)
    }

    pub fn is_time_credit_low(&self) -> bool {
        self.state() == PhoneLineState::Connected 
        && self.config.payphone.time_credit_seconds > 0
        && self.initial_deposit_consumed()
        && !self.is_current_call_free()
        && self.remaining_time_credit().as_secs() <= self.config.payphone.time_credit_warn_seconds
    }

    pub fn awaiting_initial_deposit(&self) -> bool {
        *self.awaiting_initial_deposit.borrow()
    }

    fn add_coin_deposit(&self, cents: u32) {
        let mut total = 0;
        self.coin_deposit.replace_with(|prev_cents| { total = *prev_cents + cents; total });
        info!("PBX: Deposited {}¢ (total: {}¢)", cents, total);
        self.convert_deposit_to_credit();
    }

    /// Converts deposit into time credit for the current call.
    /// Any leftover deposit will remain and count towards future credit.
    fn convert_deposit_to_credit(&self) -> bool {
        if self.state() == PhoneLineState::Connected {
            // Check if time credit can be added to the call
            let mut deposit = self.coin_deposit.borrow_mut();
            let rate = self.current_call_rate();
            if rate > 0 && *deposit >= rate {
                let rate_multiplier = *deposit / rate;
                *deposit %= rate;
                let time_credit = Duration::from_secs(self.config.payphone.time_credit_seconds.saturating_mul(rate_multiplier as u64));
                self.add_time_credit(time_credit);
                return true
            }
        }
        false
    }

    /// Adds the specified amount of time to the call (payphone only).
    fn add_time_credit(&self, credit: Duration) {
        self.time_credit.replace_with(|cur| cur.checked_add(credit).unwrap_or_default());
        info!("PBX: Added time credit: {:?}", credit);
    }

    /// Converts deposit into time credit and sets `initial_deposit_consumed` flag.
    fn consume_initial_deposit(&self) {
        self.convert_deposit_to_credit();
        self.awaiting_initial_deposit.replace(false);
        self.initial_deposit_consumed.replace(true);
        info!("PBX: Initial deposit consumed.");
    }

    /// Indicates whether the initial deposit for the current call was consumed.
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
                PhoneLineState::Idle | PhoneLineState::IdleRinging => {},
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
        use PhoneLineState::*;
        let state = self.state();
        let hook_change_time = Instant::now();
        if on_hook {
            match state {
                Idle | IdleRinging => {}
                _ => {
                    info!("PBX: Switchhook CLOSED");
                    self.set_state(PhoneLineState::Idle);
                }
            }
        } else {
            // Only process this signal if the line is inactive or ringing
            match state {
                // Picking up idle phone
                Idle => {
                    info!("PBX: Switchhook OPEN.");
                    self.set_state(PhoneLineState::DialTone);
                },
                // Answering a call
                IdleRinging => {
                    info!("PBX: Switchhook OPEN, connecting call.");
                    // Connect the call
                    self.set_state(PhoneLineState::Connected);
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
    fn update_state(&'lua self) {
        use PhoneLineState::*;
        let state = self.state();
        match state {
            DialTone => {
                let state_time = self.current_state_time();
                if state_time.as_secs_f32() >= self.config.off_hook_delay {
                    self.handle_off_hook_timeout();
                }
            }
            PDD => {
                if self.pdd_time().as_secs_f32() >= self.config.pdd {
                    match self.host_phone_type {
                        PhoneType::Payphone => {
                            let number_to_dial = self.get_dialed_number();

                            // Figure out how much the call costs
                            let price = match self.lookup_agent(number_to_dial.as_str()) {
                                Some(agent_to_call) if self.config.payphone.enable_custom_agent_rates => 
                                match agent_to_call.custom_price() {
                                    Some(cents) => cents,
                                    None => self.config.payphone.standard_call_rate
                                },
                                _ => self.config.payphone.standard_call_rate
                            };

                            // If the user has deposited enough money, call the number. Otherwise, do nothing.
                            if *self.coin_deposit.borrow() >= price {
                                self.call_number(number_to_dial.as_str());
                                self.awaiting_initial_deposit.replace(false);
                            } else {
                                self.awaiting_initial_deposit.replace(true);
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
                match self.host_phone_type {
                    PhoneType::Payphone if !self.is_current_call_free() => {
                        // Wait for user-configured delay and eat coin deposit
                        if !self.initial_deposit_consumed() {
                            if self.current_state_time() >= self.coin_consume_delay {
                                self.consume_initial_deposit();
                            }
                        } else if !self.has_time_credit() {
                            // Cut off call if time credit runs out
                            info!("Out of time credit; ending call.");
                            self.set_state(PhoneLineState::Busy);
                        }
                    },
                    _ => ()
                }                
            }
            _ => {}
        }
    }

    /// Updates the states of the agents associated with the engine.
    #[inline]
    fn update_agents(&'lua self) {
        use AgentIntent::*;
        use PhoneLineState::*;
        let state = self.state();
        let agent_modules = self.agents.borrow();
        let agent_iter = agent_modules.iter();
        for (_, agent) in agent_iter {
            let mut intent = agent.tick(AgentData::None);
            loop {
                match intent {
                    // Agent requests a digit from the user
                    Ok(ReadDigit) => {
                        if let Some(digit) = self.consume_dialed_digit() {
                            intent = agent.tick(AgentData::Digit(digit));
                            continue;
                        }
                    },
                    // Agent wants to call the user
                    Ok(CallUser) => {
                        // First, check that there's nobody on the line and the user's on-hook.
                        // Also make sure that the config allows incoming calls.
                        if self.config.features.enable_incoming_calls.unwrap_or(false) 
                        && self.state() == PhoneLineState::Idle 
                        && self.other_party.borrow().is_none() {
                            agent.set_reason(CallReason::AgentInit);
                            agent.transition_state(AgentState::OutgoingCall);
                            self.load_other_party(Rc::clone(agent));
                            self.set_state(PhoneLineState::IdleRinging);
                            self.last_caller_id.replace(agent.id());
                        } else {
                            // Tell the agent they're busy
                            intent = agent.tick(AgentData::LineBusy);
                            continue;
                        }
                    },
                    // Agent wants to accept incoming call
                    Ok(AcceptCall) => {
                        if state == CallingOut { 
                            self.set_state(Connected);
                        }
                    },
                    // Agent wants to end current call
                    Ok(EndCall) => {
                        match state {
                            // Transition to idle (hangs up at end of CALL state)
                            Connected => {
                                agent.transition_state(AgentState::Idle);
                            },
                            // Caller has given up, disconnect immediately
                            IdleRinging => {
                                agent.transition_state(AgentState::Idle);
                                self.set_state(PhoneLineState::Idle);
                            },
                            _ => {}
                        }
                    }
                    // Agent has just exited CALL state
                    Ok(StateEnded(AgentState::Call)) => {
                        // Don't affect PBX state if the call is already ended
                        match state {
                            Connected => {
                                // TODO: Allow user to customize behavior when agent ends call
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
        self.update_state();
        self.update_agents();
    }
}