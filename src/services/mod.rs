#![allow(dead_code)]
#![allow(unreachable_patterns)]

mod props;
mod api;

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

// TODO: Get rid of ServiceId fields in PbxState, as they're made redundant by PbxEngine.other_party
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PbxState {
    Idle,
    IdleRinging(ServiceId),
    DialTone,
    PDD,
    CallingOut(ServiceId),
    Connected(ServiceId),
    Busy
}

pub struct ServiceModule<'lua> {
    id: RefCell<Option<ServiceId>>,
    name: String,
    phone_number: Option<String>,
    role: ServiceRole,
    ringback_enabled: bool,
    required_sound_banks: Vec<String>,
    tbl_module: LuaTable<'lua>,
    func_load: Option<LuaFunction<'lua>>,
    func_unload: Option<LuaFunction<'lua>>,
    func_tick: LuaFunction<'lua>
}

impl<'lua> ServiceModule<'lua> {
    fn from_file(lua: &'lua Lua, path: &Path) -> Result<Self, String> {
        let src = fs::read_to_string(path).expect("Unable to read Lua source file");
        let module_chunk = lua.load(&src).set_name(path.to_str().unwrap()).unwrap();
        let module = module_chunk.eval::<LuaTable>();
        match module {
            Ok(table) => {
                let name = table.raw_get("_name").expect("Module requires a name");
                let phone_number = table.raw_get("_phone_number").unwrap();
                let role = ServiceRole::from(table.raw_get::<&'static str, usize>("_role").unwrap());
                let ringback_enabled: bool = table.raw_get("_ringback_enabled").unwrap_or(true);
                let func_load: Option<LuaFunction<'lua>> = table.raw_get("load").unwrap();
                let func_unload = table.raw_get("unload").unwrap();
                let func_tick = table.get("tick").expect("tick() function not found");
                let mut required_sound_banks: Vec<String> = Default::default();

                // Start state machine
                table.call_method::<&str, _, ()>("start", ()).expect(format!("Unable to start state machine for {}", name).as_str());

                // Call load() if available
                if let Some(func_load) = &func_load {
                    let load_args = lua.create_table().unwrap();
                    load_args.set("path", path.to_str()).unwrap();
                    if let Err(err) = func_load.call::<LuaTable, ()>(load_args) {
                        return Err(format!("Error while calling service loader: {:#?}", err));
                    }
                }      
                
                // Get required sound banks
                if let Ok(bank_name_table) = table.raw_get::<&'static str, LuaTable>("_required_sound_banks") {
                    let pairs = bank_name_table.pairs::<String, bool>();
                    for pair in pairs {
                        if let Ok((bank_name, required)) = pair {
                            if !required || bank_name.is_empty() { continue }
                            required_sound_banks.push(bank_name);
                        }
                    }
                }

                Ok(Self {
                    id: Default::default(),
                    required_sound_banks,
                    ringback_enabled,
                    tbl_module: table,
                    name,
                    role,
                    phone_number,
                    func_load,
                    func_unload,
                    func_tick,
                })
            },
            Err(err) => Err(format!("Unable to load service module: {:#?}", err))
        }
    }

    fn load_sound_banks(&self, sound_engine: &Rc<RefCell<SoundEngine>>) {
        let mut sound_engine = sound_engine.borrow_mut();
        for bank_name in &self.required_sound_banks {
            sound_engine.add_sound_bank_user(bank_name, SoundBankUser(self.id().unwrap()));
        }
    }

    fn unload_sound_banks(&self, sound_engine: &Rc<RefCell<SoundEngine>>) {
        let mut sound_engine = sound_engine.borrow_mut();
        for bank_name in &self.required_sound_banks {
            sound_engine.remove_sound_bank_user(bank_name, SoundBankUser(self.id().unwrap()), true);
        }
    }

    fn register_id(&self, id: ServiceId) {
        self.id.replace(Some(id));
    }

    pub fn id(&self) -> Option<ServiceId> {
        *self.id.borrow()
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn suspended(&self) -> bool {
        self.tbl_module.get("_is_suspended").unwrap_or(false)
    }

    pub fn set_reason(&self, reason: CallReason) -> LuaResult<()> {
        self.tbl_module.call_method("set_reason", reason.as_index())?;
        Ok(())
    }

    pub fn state(&self) -> LuaResult<ServiceState> {
        let raw_state = self.tbl_module.get::<&str, usize>("_state")?;
        Ok(ServiceState::from(raw_state))
    }

    #[inline]
    fn tick(&self, data: ServiceData) -> LuaResult<ServiceIntent> {
        if self.suspended() {
            return Ok(ServiceIntent::Idle)
        }

        let service_table = self.tbl_module.clone();
        let data_code = data.to_code();

        // Tick service
        let (intent_code, intent_data) = match data {
            ServiceData::None => self.func_tick.call((service_table, data_code))?,
            ServiceData::Digit(digit) => self.func_tick.call((service_table, data_code, digit.to_string()))?,
            ServiceData::LineBusy => self.func_tick.call((service_table, data_code))?
        };

        let intent = ServiceIntent::from_lua_value(intent_code, intent_data);
        Ok(intent)
    }

    fn transition_state(&self, state: ServiceState) -> LuaResult<()> {
        self.tbl_module.call_method("transition", state.as_index())?;
        Ok(())
    }
}

/// A Lua-powered telephone exchange that loads,
/// manages, and runs scripted phone services.
pub struct PbxEngine<'lua> {
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
    /// Is host rotary dial resting?
    host_rotary_resting: RefCell<bool>,
    /// Number of host pulses since last dialed digit.
    host_rotary_pulses: RefCell<usize>,
    /// Time of the last lifting of the rotary dial rest switch.
    host_rotary_dial_lift_time: RefCell<Instant>,
    /// Delay between rotary dial leaving resting state and first valid pulse.
    host_rotary_first_pulse_delay: Duration,
}

impl<'lua> Drop for ServiceModule<'lua> {
    fn drop(&mut self) {
        if let Some(unload) = &self.func_unload {
            if let Err(error) = unload.call::<(), ()>(()) {
                error!("Service module '{}' encountered error while unloading: {:?}", self.name, error);
            }
        }
    }
}

#[allow(unused_must_use)]
impl<'lua> PbxEngine<'lua> {
    pub fn new(scripts_root: impl Into<String>, config: &Rc<CursedConfig>, sound_engine: &Rc<RefCell<SoundEngine>>) -> Self {
        let lua = Lua::new();
        let now = Instant::now();
        Self {
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
            host_rotary_pulses: Default::default(),
            host_rotary_resting: Default::default(),
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
                info!("PBX: Connecting call -> {} ({:?})", service.name, service.phone_number);
                self.set_state(CallingOut(service.id().unwrap()));
            },
            _ => {}
        }
    }

    /// Calls the intercept service, if available.
    fn call_intercept(&'lua self, reason: CallReason) {
        if let Some(intercept_service) = self.intercept_service.borrow().as_ref() {
            intercept_service.set_reason(reason);
            self.call_service(Rc::clone(intercept_service));
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
                        let service_name = service.name.clone();
                        let service_role = service.role;
                        let service_phone_number = service.phone_number.clone();
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

                        info!("Service loaded: {} (N = {:?}, ID = {:?})", service.name, service.phone_number, service.id());
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
            PbxState::IdleRinging(_) => {
                self.send_output(PhoneOutputSignal::Ring(false));
            },
            _ => {}
        }

        // Run behavior for new state
        match state {
            Idle | IdleRinging(_) => {},
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
            (_, IdleRinging(id)) => {
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
            (_, CallingOut(id)) => {
                self.clear_dialed_number();
                self.sound_engine.borrow().stop(Channel::SignalIn);

                // Set other_party to requested service
                let service = self.lookup_service_id(id).unwrap();
                self.load_other_party(Rc::clone(&service));

                // Tell service that we're calling it
                service.transition_state(ServiceState::IncomingCall);

                // Finally, play the ringback tone (if we're allowed to)
                if service.ringback_enabled {
                    self.sound_engine.borrow().play_ringback_tone();
                }
            },
            (_, Connected(_)) => {
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
    fn handle_off_hook_timeout(&'lua self) {
        info!("PBX: Off-hook timeout.");
        self.call_intercept(CallReason::OffHook);
    }

    /// Called when the host dials a digit via any method.
    fn handle_host_digit(&'lua self, digit: char) {
        use PbxState::*;

        // Perform special digit-triggered behaviors
        match self.state() {
            // Ignore digits dialed while the phone is on the hook
            Idle | IdleRinging(_) => return,

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
    fn handle_rotary_pulse(&self) {
        match self.state() {
            PbxState::Idle | PbxState::IdleRinging(_) => return,
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

    /// Called when the resting state of the host's rotary dial changes.
    fn handle_rotary_rest_state(&'lua self, resting: bool) {
        if resting == self.host_rotary_resting.replace(resting) {return}

        if resting {
            // Ignore 
            match self.state() {
                PbxState::Idle | PbxState::IdleRinging(_) => {},
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

    /// Reads and handles pending input signals from the host device.
    fn process_input_signals(&'lua self) {
        if let Some(phone_input) = self.phone_input.borrow().as_ref() {
            while let Ok(signal) = phone_input.try_recv() {
                let state = self.state();
                use PhoneInputSignal::*;
                use PbxState::*;
                // TODO: Move switchhook dialing to PBX?
                match signal {
                    HookState(true) => {
                        match state {
                            Idle | IdleRinging(_) => {}
                            _ => {
                                info!("PBX: Host on-hook.");
                                self.set_state(PbxState::Idle);
                            }
                        }
                    },
                    HookState(false) => {
                        // Only process this signal if the line is inactive or ringing
                        match state {
                            // Picking up idle phone
                            Idle => {
                                info!("PBX: Host off-hook.");
                                self.set_state(PbxState::DialTone);
                            },
                            // Answering a call
                            IdleRinging(id) => {
                                info!("PBX: Host off-hook, connecting call.");
                                // Connect the call
                                self.set_state(PbxState::Connected(id));
                            },
                            _ => {}
                        }
                    },
                    RotaryDialRest(resting) => self.handle_rotary_rest_state(resting),
                    RotaryDialPulse => self.handle_rotary_pulse(),
                    Motion => {
                        info!("PBX: Detected motion.");
                    },
                    Digit(digit) => {
                        self.handle_host_digit(digit);
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
                    let number_to_dial = self.get_dialed_number();
                    self.call_number(number_to_dial.as_str());
                }
            },
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
                        // First, check that there's nobody on the line and the user's on-hook
                        if self.state() == PbxState::Idle && self.other_party.borrow().is_none() {
                            let id = service.id().unwrap();
                            service.set_reason(CallReason::ServiceInit);
                            service.transition_state(ServiceState::OutgoingCall);
                            self.load_other_party(Rc::clone(service));
                            self.set_state(PbxState::IdleRinging(id));
                        } else {
                            // Tell the service they're busy
                            intent = service.tick(ServiceData::LineBusy);
                            continue;
                        }
                    },
                    // Service wants to accept incoming call
                    Ok(AcceptCall) => {
                        let id = service.id().unwrap();
                        if state == CallingOut(id) {                
                            service.set_reason(CallReason::UserInit);            
                            self.set_state(Connected(id));
                        }
                    },
                    // Service wants to end current call
                    Ok(EndCall) => {
                        let id = service.id().unwrap();
                        match state {
                            // Transition to idle (hangs up at end of CALL state)
                            Connected(id) => {
                                service.transition_state(ServiceState::Idle);
                            },
                            // Caller has given up, disconnect immediately
                            IdleRinging(id) => {
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
                            Connected(id) if id == service.id().unwrap() => {
                                self.set_state(Busy);
                            },
                            _ => {}
                        }
                    },
                    Ok(intent) => {
                        
                    },
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