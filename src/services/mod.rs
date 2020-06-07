#![allow(dead_code)]

mod props;
mod api;

use std::rc::Rc;
use std::cell::RefCell;
use std::{thread, time, fs, sync::mpsc};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::time::Instant;
use rand::Rng;
use mlua::prelude::*;
use indexmap::IndexMap;
use crate::sound::*;
use crate::phone::*;

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
    services: RefCell<IndexMap<String, ServiceModule<'lua>>>,
    /// The sound engine associated with the engine.
    sound_engine: RcRefCell<SoundEngine>,
    /// The intercept service.
    intercept_service: Option<ServiceModule<'lua>>,
    /// Channel for sending output signals to the host phone.
    phone_output: RefCell<Option<mpsc::Sender<PhoneOutputSignal>>>,
    /// Channel for receiving input signals from the host phone.
    phone_input: RefCell<Option<mpsc::Receiver<PhoneInputSignal>>>,
    /// The current state of the engine.
    state: RefCell<PbxState>
}

pub struct ServiceModule<'lua> {
    id: ServiceId,
    name: String,
    phone_number: Option<String>,
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
                    id: 0,
                    ringback_enabled,
                    tbl_module: table,
                    name,
                    phone_number,
                    func_load,
                    func_unload,
                    func_tick
                })
            },
            Err(err) => Err(format!("Unable to load service module: {:#?}", err))
        }
    }

    fn set_id(&mut self, id: usize) {
        self.id = id;
    }

    pub fn id(&self) -> usize {
        self.id
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
    fn tick(&self) -> LuaResult<()> {
        if self.suspended() {
            return Ok(())
        }

        let (status_code, status_data) = self.func_tick.call(self.tbl_module.clone())?;
        let status = ServiceIntent::from_lua_value(status_code, status_data);
        use ServiceIntent::*;
        match status {   
            Idle|Wait => {},
            // TODO
            CallUser => {},
            // TODO
            EndCall => {},
            // TODO
            AcceptCall => {},
            // TODO
            RequestDigit => {},
            // TODO
            ForwardCall(number) => {}
            StateEnd(next_state) => {
                self.transition_state(next_state)?;
            }
        }
        Ok(())
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
    pub fn new(scripts_root: impl Into<String>, sound_engine: &Rc<RefCell<SoundEngine>>) -> Self {
        let lua = Lua::new();
        Self {
            lua,
            start_time: Instant::now(),
            scripts_root: Path::new(scripts_root.into().as_str()).canonicalize().unwrap(),
            sound_engine: Rc::clone(sound_engine),
            phone_book: Default::default(),
            services: Default::default(),
            intercept_service: None,
            state: RefCell::new(PbxState::Idle),
            phone_input: Default::default(),
            phone_output: Default::default()
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

    fn set_state(&self, state: PbxState) {
        if *self.state.borrow() == state {
            return;
        }

        let prev_state = self.state.replace(state);
        use PbxState::*;
        match (prev_state, state) {
            // TODO
            (_, Idle) => {},
            _ => {}
        }
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
                let service_module = ServiceModule::from_file(&self.lua, &module_path);
                match service_module {
                    Ok(service_module) => {
                        println!("Service loaded: {} (N = {:?}, ID = {})", service_module.name, service_module.phone_number, service_module.id);

                        // Register service
                        let service_phone_number = service_module.phone_number.clone();
                        let (service_id, _) = services.insert_full(service_module.name.clone(), service_module);
                        services.get_index_mut(service_id).unwrap().1.set_id(service_id);

                        // Register service number
                        if let Some(phone_number) = service_phone_number {
                            if !phone_number.is_empty() {
                                services_numbered.insert(phone_number, service_id);
                            }
                        }
                    },
                    Err(err) => {
                        println!("Failed to load service module '{:?}': {:#?}", module_path, err);
                    }
                }
            }
        }
    }

    fn process_input_signals(&self) {
        if let Some(phone_input) = self.phone_input.borrow().as_ref() {
            while let Ok(signal) = phone_input.try_recv() {
                use PhoneInputSignal::*;
                match signal {
                    // TODO
                    HookState(on_hook) => {
                        println!("PBX: Received hook state ({})", on_hook);
                    },
                    Motion => {
                        println!("PBX: Received motion");
                    },
                    Digit(digit) => {
                        println!("PBX: Received digit '{}'", digit);
                    }
                }
            }
        }
    }

    #[inline]
    fn update_services(&self) {
        let service_modules = self.services.borrow();
        let service_iter = service_modules.iter();
        for (_, service) in service_iter {
            if let Err(err) = service.tick() {
                self.sound_engine.borrow().play_panic_tone();
                match err {
                    LuaError::RuntimeError(msg) => println!("LUA ERROR: {}", msg),
                    _ => println!("LUA ERROR: {:?}", err)
                }
            }
        }
    }

    pub fn tick(&self) {
        self.process_input_signals();
        self.update_services();
    }
}