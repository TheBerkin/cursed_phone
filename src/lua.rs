#![allow(dead_code)]

use crate::sound::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::{thread, time};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use rand::Rng;
use indexmap::IndexMap;
use mlua::prelude::*;

const BOOTSTRAPPER_SCRIPT_NAME: &str = "bootstrapper";
const API_GLOB: &str = "api/*";

pub struct LuaEngine<'lua> {
    lua: Lua,
    scripts_root: PathBuf,
    start_time: Instant,
    service_modules: RefCell<IndexMap<String, ServiceModule<'lua>>>,
    sound_engine: Rc<RefCell<SoundEngine>>
}

#[derive(Clone, Debug)]
enum ServiceIntent {
    Idle,
    AcceptCall,
    EndCall,
    CallUser,
    Waiting,
    RequestDigit,
    Forward(String),
    FinishedState(ServiceState)
}

impl ServiceIntent {
    fn from_lua_value(status_code: i32, status_data: LuaValue) -> ServiceIntent {
        match status_code {
            0 => ServiceIntent::Idle,
            1 => ServiceIntent::AcceptCall,
            2 => ServiceIntent::EndCall,
            3 => ServiceIntent::CallUser,
            4 => ServiceIntent::Waiting,
            5 => ServiceIntent::RequestDigit,
            6 => match status_data {
                LuaValue::String(s) => ServiceIntent::Forward(String::from(s.to_str().unwrap())),
                _ => ServiceIntent::Forward(String::from("A"))
            },
            7 => match status_data {
                LuaValue::Integer(n) => ServiceIntent::FinishedState(ServiceState::from(n as usize)),
                _ => ServiceIntent::Idle
            },
            _ => ServiceIntent::Idle
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum ServiceState {
    /// Service is currently idle.
    Idle = 0,
    /// Service is placing a call.
    OutgoingCall = 1,
    /// Service is receiving a call.
    IncomingCall = 2,
    /// Service is in a call.
    Call = 3
}

const ALL_SERVICE_STATES: &[ServiceState] = { use ServiceState::*; &[Idle, OutgoingCall, IncomingCall, Call] };

impl From<usize> for ServiceState {
    fn from(value: usize) -> ServiceState {
        ALL_SERVICE_STATES[value]
    }
}

impl ServiceState {
    fn as_index(self) -> usize {
        self as usize
    }
}

#[derive(Copy, Clone, Debug)]
enum ServiceRole {
    Normal = 0,
    Intercept = 1
}

const ALL_SERVICE_ROLES: &[ServiceRole] = { use ServiceRole::*; &[Normal, Intercept] };

impl From<usize> for ServiceRole {
    fn from(value: usize) -> ServiceRole {
        ALL_SERVICE_ROLES[value]
    }
}

struct ServiceModule<'lua> {
    name: String,
    phone_number: Option<String>,
    tbl_module: LuaTable<'lua>,
    func_load: Option<LuaFunction<'lua>>,
    func_unload: Option<LuaFunction<'lua>>
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
                let func_load: Option<LuaFunction<'lua>> = table.raw_get("load").unwrap();
                let func_unload = table.raw_get("unload").unwrap();

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
                    tbl_module: table,
                    name,
                    phone_number,
                    func_load,
                    func_unload
                })
            },
            Err(err) => Err(format!("Unable to load service module: {:#?}", err))
        }
    }

    fn state(&self) -> LuaResult<ServiceState> {
        let raw_state = self.tbl_module.get::<&str, usize>("_state")?;
        Ok(ServiceState::from(raw_state))
    }

    #[inline]
    fn tick(&self) -> LuaResult<()> {
        let (status_code, status_data) = self.tbl_module.call_method("tick", ())?;
        let status = ServiceIntent::from_lua_value(status_code, status_data);
        use ServiceIntent::*;
        match status {   
            Idle|Waiting => {},
            // TODO
            CallUser => {},
            // TODO
            EndCall => {},
            // TODO
            AcceptCall => {},
            // TODO
            RequestDigit => {},
            // TODO
            Forward(number) => {}
            FinishedState(next_state) => {
                self.set_state(next_state)?;
            }
        }
        Ok(())
    }

    fn set_state(&self, state: ServiceState) -> LuaResult<()> {
        self.tbl_module.call_method("set_state", state.as_index())?;
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
impl<'lua> LuaEngine<'lua> {
    pub fn new(scripts_root: impl Into<String>, sound_engine: &Rc<RefCell<SoundEngine>>) -> Self {
        let lua = Lua::new();
        Self {
            lua,
            start_time: Instant::now(),
            scripts_root: Path::new(scripts_root.into().as_str()).canonicalize().unwrap(),
            sound_engine: Rc::clone(sound_engine),
            service_modules: Default::default()
        }
    }

    fn lua_sleep(_: &Lua, ms: u64) -> LuaResult<()> {
        thread::sleep(time::Duration::from_millis(ms));
        Ok(())
    }

    fn lua_random_int(_: &Lua, (min, max): (i32, i32)) -> LuaResult<i32> {
        if min >= max {
            return Ok(min);
        }
        Ok(rand::thread_rng().gen_range(min, max))
    }

    fn lua_random_float(_: &Lua, (min, max): (f64, f64)) -> LuaResult<f64> {
        if min >= max {
            return Ok(min);
        }
        Ok(rand::thread_rng().gen_range(min, max))
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

    pub fn load_cursed_api(&'static self) -> Result<(), String> {
        // Run bootstrapper script
        println!("Bootstrapping Lua...");
        self.run_script(BOOTSTRAPPER_SCRIPT_NAME)?;

        let lua = &self.lua;
        let globals = &lua.globals();

        // ====================================================
        // ==================== SOUND API =====================
        // ====================================================

        let tbl_sound = lua.create_table().unwrap();    

        // sound.play(path, channel, opts)
        tbl_sound.set("play", lua.create_function(move |_, (path, channel, opts): (String, usize, Option<LuaTable>)| {
            let mut speed: Option<f32> = None;
            let mut interrupt: Option<bool> = None;
            let mut looping: Option<bool> = None;
            let mut volume: Option<f32> = None;
            match opts {
                Some(opts_table) => {
                    speed = opts_table.get::<&str, f32>("speed").ok();
                    interrupt = opts_table.get::<&str, bool>("interrupt").ok();
                    looping = opts_table.get::<&str, bool>("looping").ok();
                    volume = opts_table.get::<&str, f32>("volume").ok();
                },
                None => {}
            }
            self.sound_engine.borrow().play(
                path.as_str(), 
                Channel::from(channel), 
                false, 
                looping.unwrap_or(false), 
                interrupt.unwrap_or(true), 
                speed.unwrap_or(1.0),
                volume.unwrap_or(1.0)
            );
            Ok(())
        }).unwrap());

        // sound.is_busy(channel)
        tbl_sound.set("is_busy", lua.create_function(move |_, channel: usize| {
            let busy = self.sound_engine.borrow().channel_busy(Channel::from(channel));
            Ok(busy)
        }).unwrap());

        // sound.stop(channel)
        tbl_sound.set("stop", lua.create_function(move |_, channel: usize| {
            self.sound_engine.borrow().stop(Channel::from(channel));
            Ok(())
        }).unwrap());

        // sound.stop_all(channel)
        tbl_sound.set("stop_all", lua.create_function(move |_, ()| {
            self.sound_engine.borrow().stop_all();
            Ok(())
        }).unwrap());

        // sound.get_channel_volume(channel)
        tbl_sound.set("get_channel_volume", lua.create_function(move |_, channel: usize| {
            let vol = self.sound_engine.borrow().volume(Channel::from(channel));
            Ok(vol)
        }).unwrap());

        // sound.set_channel_volume(channel, volume)
        tbl_sound.set("set_channel_volume", lua.create_function(move |_, (channel, volume): (usize, f32)| {
            self.sound_engine.borrow_mut().set_volume(Channel::from(channel), volume);
            Ok(())
        }).unwrap());

        // sound.play_dial_tone()
        tbl_sound.set("play_dial_tone", lua.create_function(move |_, ()| {
            self.sound_engine.borrow().play_dial_tone();
            Ok(())
        }).unwrap());

        // sound.play_busy_tone()
        tbl_sound.set("play_busy_tone", lua.create_function(move |_, ()| {
            self.sound_engine.borrow().play_busy_tone();
            Ok(())
        }).unwrap());

        // sound.play_ringback_tone()
        tbl_sound.set("play_ringback_tone", lua.create_function(move |_, ()| {
            self.sound_engine.borrow().play_ringback_tone();
            Ok(())
        }).unwrap());

        // sound.play_dtmf_digit(digit, duration, volume)
        tbl_sound.set("play_dtmf_digit", lua.create_function(move |_, (digit, duration, volume): (u8, f32, f32)| {
            self.sound_engine.borrow().play_dtmf(digit as char, duration, volume);
            Ok(())
        }).unwrap());

        globals.set("sound", tbl_sound);

        // ====================================================
        // ============ MISC NATIVE API FUNCTIONS =============
        // ====================================================

        // sleep()
        globals.set("sleep", lua.create_function(LuaEngine::lua_sleep).unwrap());
        // random_int()
        globals.set("random_int", lua.create_function(LuaEngine::lua_random_int).unwrap());
        // random_float()
        globals.set("random_float", lua.create_function(LuaEngine::lua_random_float).unwrap());

        // get_run_time()
        globals.set("get_run_time", lua.create_function(move |_, ()| {
            let run_time = self.start_time.elapsed().as_secs_f64();
            Ok(run_time)
        }).unwrap());

        // Run API scripts
        self.run_scripts_in_glob(API_GLOB)?;

        Ok(())
    }

    pub fn load_services(&'lua self) {
        self.service_modules.borrow_mut().clear();
        let search_path = self.scripts_root.join("services").join("**").join("*.lua");
        let search_path_str = search_path.to_str().expect("Failed to create search pattern for service modules");
        for entry in globwalk::glob(search_path_str).expect("Unable to read search pattern for service modules") {
            if let Ok(dir) = entry {
                let module_path = dir.path().canonicalize().expect("Unable to expand service module path");
                let service_module = ServiceModule::from_file(&self.lua, &module_path);
                match service_module {
                    Ok(service_module) => {
                        if let Some(key) = service_module.phone_number.clone() {
                            println!("Service loaded: {}", service_module.name);
                            self.service_modules.borrow_mut().insert(key, service_module);
                        } else {
                            // TODO: Handle number-less services
                        }
                    },
                    Err(err) => {
                        println!("Failed to load service module '{:?}': {:#?}", module_path, err);
                    }
                }
            }
        }
    }

    pub fn tick(&self) {
        let service_modules = self.service_modules.borrow();
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
}