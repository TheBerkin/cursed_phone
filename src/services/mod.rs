#![allow(dead_code)]

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
use crate::sound::*;
use crate::phone::*;
use crate::config::*;

pub use self::api::*;
pub use self::props::*;

/// `Option<Rc<T>>`
type Orc<T> = Option<Rc<T>>;

/// `Rc<RefCell<T>>`
type RcRefCell<T> = Rc<RefCell<T>>;

const BOOTSTRAPPER_SCRIPT_NAME: &str = "bootstrapper";
const API_GLOB: &str = "api/*";

type ServiceId = usize;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PbxState {
    Idle,
    DialTone,
    PDD,
    CallingOut(ServiceId),
    CallingHost(ServiceId),
    Connected(ServiceId),
    Busy
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
}

pub struct ServiceModule<'lua> {
    id: RefCell<Option<ServiceId>>,
    name: String,
    phone_number: Option<String>,
    role: ServiceRole,
    ringback_enabled: bool,
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

                Ok(Self {
                    id: Default::default(),
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

    pub fn state(&self) -> LuaResult<ServiceState> {
        let raw_state = self.tbl_module.get::<&str, usize>("_state")?;
        Ok(ServiceState::from(raw_state))
    }

    #[inline]
    fn tick(&self) -> LuaResult<ServiceIntent> {
        if self.suspended() {
            return Ok(ServiceIntent::Idle)
        }

        let (intent_code, intent_data) = self.func_tick.call(self.tbl_module.clone())?;
        let intent = ServiceIntent::from_lua_value(intent_code, intent_data);
        Ok(intent)
    }

    fn transition_state(&self, state: ServiceState) -> LuaResult<()> {
        self.tbl_module.call_method("transition", state.as_index())?;
        Ok(())
    }
}

impl<'lua> Drop for ServiceModule<'lua> {
    fn drop(&mut self) {
        if let Some(unload) = &self.func_unload {
            if let Err(error) = unload.call::<(), ()>(()) {
                println!("Service module '{}' encountered error while unloading: {:?}", self.name, error);
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

    fn lookup_service_id(&self, id: ServiceId) -> Orc<ServiceModule> {
        self.services.borrow().get_index(id).map(|result| Rc::clone(result.1))
    }

    fn lookup_service(&self, phone_number: &str) -> Orc<ServiceModule> {
        if let Some(id) = self.phone_book.borrow().get(phone_number) {
            return self.lookup_service_id(*id);
        }
        None
    }

    fn call_number(&'lua self, number: &str) -> bool {
        println!("Placing call to: {}", number);
        if let Some(service) = self.lookup_service(number) {
            self.call_service(service);
            return true;
        } else {
            self.call_intercept();
            return false;
        }
    }

    fn call_service(&'lua self, service: Rc<ServiceModule>) {
        use PbxState::*;
        match self.state() {
            DialTone | Busy | PDD => {
                println!("PBX: Connecting call -> {} ({:?})", service.name, service.phone_number);
                self.set_state(CallingOut(service.id().unwrap()));
            },
            _ => {}
        }
    }

    fn call_intercept(&'lua self) {
        if let Some(intercept_service) = self.intercept_service.borrow().as_ref() {
            self.call_service(Rc::clone(intercept_service));
        } else {
            // Default to busy signal if there is no intercept service
            println!("PBX: No intercept service; defaulting to busy signal.");
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
        //println!("PBX: Dialed number cleared.");
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
                println!("Loading API: {}", script_path_str);
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
                        println!("Service loaded: {} (N = {:?}, ID = {:?})", service_name, service_phone_number, service.id());
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
                    },
                    Err(err) => {
                        println!("Failed to load service module '{:?}': {:#?}", module_path, err);
                    }
                }
            }
        }
    }

    fn get_other_party_service(&self) -> Orc<ServiceModule> {
        if let Some(service) = self.other_party.borrow().as_ref() {
            return Some(Rc::clone(service));
        }
        None
    }

    fn set_state(&'lua self, state: PbxState) {
        if *self.state.borrow() == state {
            return;
        }

        let prev_state = self.state.replace(state);
        let state_start = Instant::now();
        let last_state_start = self.state_start.replace(state_start);
        let state_time = state_start.saturating_duration_since(last_state_start);

        // Run behavior for state transition
        use PbxState::*;
        match (prev_state, state) {
            // TODO: Implement all valid PBX state transitions
            (_, Idle) => {
                // Force other party (if available) to idle state
                if let Some(service) = self.get_other_party_service() {
                    service.transition_state(ServiceState::Idle);
                    self.other_party.replace(None);
                }
                self.sound_engine.borrow().stop_all_except(Channel::SignalOut);
                self.clear_dialed_number();
            },
            (_, DialTone) => {
                self.sound_engine.borrow().play_dial_tone();
            },
            (_, Busy) => {
                self.other_party.replace(None);
                let sound_engine = self.sound_engine.borrow();
                sound_engine.stop_all_except(Channel::SignalOut);
                sound_engine.play_busy_tone();
            },
            (_, PDD) => {
                // Stop any PBX signals
                self.sound_engine.borrow().stop(Channel::SignalIn);
                self.update_pdd_start();
                // TODO: Wait for digits
            },
            (_, CallingOut(id)) => {
                self.clear_dialed_number();
                let sound_engine = self.sound_engine.borrow();
                sound_engine.stop(Channel::SignalIn);

                // Set other_party to requested service
                let service = self.lookup_service_id(id).unwrap();
                self.other_party.replace(Some(Rc::clone(&service)));

                // Tell service that we're calling it
                service.transition_state(ServiceState::IncomingCall);

                // Finally, play the ringback tone (if we're allowed to)
                if service.ringback_enabled {
                    sound_engine.play_ringback_tone();
                }
            },
            (_, Connected(id)) => {
                self.clear_dialed_number();
                let sound_engine = self.sound_engine.borrow();
                sound_engine.stop(Channel::SignalIn);
            }
            _ => {}
        }

        println!("PBX: {:?} -> {:?} ({:?})", prev_state, state, state_time);
    }

    fn handle_off_hook_timeout(&'lua self) {
        println!("PBX: Off-hook timeout.");
        self.call_intercept();
    }

    fn handle_digit(&'lua self, digit: char) {
        use PbxState::*;
        println!("PBX: Digit '{}'", digit);
        let state = self.state();
        match state {
            Idle => return,
            DialTone => {
                self.set_state(PbxState::PDD);
            },
            PDD => {
                self.update_pdd_start();
            },
            _ => {} // TODO: Pass digits to service in call
        }

        // Add digit to dialed number
        self.dialed_number.borrow_mut().push(digit);
    }

    fn process_input_signals(&'lua self) {
        if let Some(phone_input) = self.phone_input.borrow().as_ref() {
            while let Ok(signal) = phone_input.try_recv() {
                let state = self.state();
                use PhoneInputSignal::*;
                use PbxState::*;
                // TODO: Move switchhook dialing to PBX?
                match signal {
                    HookState(true) => {
                        if state != Idle {
                            println!("PBX: Interrupted.");
                            self.set_state(PbxState::Idle);
                        }
                    },
                    HookState(false) => {
                        // Only process this signal if the line is inactive
                        if state == Idle {
                            println!("PBX: Connected.");
                            self.set_state(PbxState::DialTone);
                        }
                    },
                    Motion => {
                        println!("PBX: Detected motion.");
                    },
                    Digit(digit) => {
                        self.handle_digit(digit);
                    }
                }
            }
        }
    }

    #[inline]
    fn current_state_time(&self) -> Duration {
        Instant::now().saturating_duration_since(*self.state_start.borrow())
    }

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

    #[inline]
    fn update_services(&'lua self) {
        use ServiceIntent::*;
        use PbxState::*;
        let state = self.state();
        let service_modules = self.services.borrow();
        let service_iter = service_modules.iter();
        for (_, service) in service_iter {
            match service.tick() {
                Ok(AcceptCall) => {
                    let id = service.id().unwrap();
                    if state == CallingOut(id) {
                        service.transition_state(ServiceState::Call);
                        self.set_state(Connected(id));
                    }
                },
                Ok(EndCall) => {
                    let id = service.id().unwrap();
                    if state == Connected(id) {
                        service.transition_state(ServiceState::Idle);
                    }
                }
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
                        LuaError::RuntimeError(msg) => println!("LUA ERROR: {}", msg),
                        _ => println!("LUA ERROR: {:?}", err)
                    }
                }
            }
        }
    }

    pub fn tick(&'lua self) {
        self.process_input_signals();
        self.update_pbx_state();
        self.update_services();
    }
}