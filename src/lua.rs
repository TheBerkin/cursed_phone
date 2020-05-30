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

const API_SCRIPT_NAME: &str = "api";

pub struct LuaEngine<'lua> {
    lua: Lua,
    scripts_root: PathBuf,
    start_time: Instant,
    service_modules: RefCell<IndexMap<String, PhoneServiceModule<'lua>>>,
    sound_engine: Rc<RefCell<SoundEngine>>
}

#[derive(Clone, Debug)]
enum PhoneServiceStatus {
    Idle,
    AcceptCall,
    EndCall,
    CallUser,
    Waiting,
    RequestDigit,
    Forward(String),
    FinishedState(PhoneServiceState)
}

impl PhoneServiceStatus {
    fn from_lua(status_code: i32, status_data: LuaValue) -> PhoneServiceStatus {
        match status_code {
            0 => PhoneServiceStatus::Idle,
            1 => PhoneServiceStatus::AcceptCall,
            2 => PhoneServiceStatus::EndCall,
            3 => PhoneServiceStatus::CallUser,
            4 => PhoneServiceStatus::Waiting,
            5 => PhoneServiceStatus::RequestDigit,
            6 => match status_data {
                LuaValue::String(s) => PhoneServiceStatus::Forward(String::from(s.to_str().unwrap())),
                _ => PhoneServiceStatus::Forward(String::from("A"))
            },
            7 => match status_data {
                LuaValue::Integer(n) => PhoneServiceStatus::FinishedState(PhoneServiceState::from(n as usize)),
                _ => PhoneServiceStatus::Idle
            },
            _ => PhoneServiceStatus::Idle
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum PhoneServiceState {
    Idle = 0,
    OutgoingCall = 1,
    IncomingCall = 2,
    Call = 3
}

const ALL_SERVICE_STATES: &[PhoneServiceState] = { use PhoneServiceState::*; &[Idle, OutgoingCall, IncomingCall, Call] };

impl From<usize> for PhoneServiceState {
    fn from(value: usize) -> PhoneServiceState {
        ALL_SERVICE_STATES[value]
    }
}

impl PhoneServiceState {
    fn as_index(self) -> usize {
        self as usize
    }
}

struct PhoneServiceModule<'lua> {
    name: String,
    phone_number: Option<String>,
    tbl_module: LuaTable<'lua>,
    func_load: Option<LuaFunction<'lua>>,
    func_unload: Option<LuaFunction<'lua>>
}

impl<'lua> PhoneServiceModule<'lua> {
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
                    func_unload // TODO: Call this in destructor
                })
            },
            Err(err) => Err(format!("Unable to load service module: {:#?}", err))
        }
    }

    fn state(&self) -> LuaResult<PhoneServiceState> {
        let raw_state = self.tbl_module.get::<&str, usize>("_state")?;
        Ok(PhoneServiceState::from(raw_state))
    }

    #[inline]
    fn tick(&self) -> LuaResult<()> {
        let (status_code, status_data) = self.tbl_module.call_method("tick", ())?;
        let status = PhoneServiceStatus::from_lua(status_code, status_data);
        use PhoneServiceStatus::*;
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

    fn set_state(&self, state: PhoneServiceState) -> LuaResult<()> {
        self.tbl_module.call_method("set_state", state.as_index())?;
        Ok(())
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
            Ok(lua_src) => self.lua.load(&lua_src).set_name("api").unwrap().exec(),
            Err(err) => return Err(format!("Failed to run lua file '{}': {:#?}", path.to_str().unwrap(), err))
        };

        Ok(())
    }

    fn resolve_script_path(&self, name: &str) -> PathBuf {
        self.scripts_root.join(name).with_extension("lua")
    }

    pub fn load_cursed_api(&'static self) -> Result<(), String> {
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
            match opts {
                Some(opts_table) => {
                    speed = opts_table.get::<&str, f32>("speed").ok();
                    interrupt = opts_table.get::<&str, bool>("interrupt").ok();
                    looping = opts_table.get::<&str, bool>("looping").ok();
                },
                None => {}
            }
            self.sound_engine.borrow().play(
                path.as_str(), 
                Channel::from(channel), 
                false, 
                looping.unwrap_or(false), 
                interrupt.unwrap_or(true), 
                speed.unwrap_or(1.0));
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

        // sound.wait(channel)
        tbl_sound.set("wait", lua.create_function(move |_, channel: usize| {
            self.sound_engine.borrow().wait(Channel::from(channel));
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

        // Load/run API script
        self.run_script(API_SCRIPT_NAME)?;

        Ok(())
    }

    pub fn load_services(&'lua self) {
        self.service_modules.borrow_mut().clear();
        let search_path = self.scripts_root.join("services").join("**").join("*.lua");
        let search_path_str = search_path.to_str().expect("Failed to create search pattern for service modules");
        for entry in globwalk::glob(search_path_str).expect("Unable to read search pattern for service modules") {
            if let Ok(path) = entry {
                let module_path = path.path().canonicalize().expect("Unable to expand service module path");
                let service_module = PhoneServiceModule::from_file(&self.lua, &module_path);
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
                println!("LUA ERROR: {:#?}", err);
            }
        }
    }
}