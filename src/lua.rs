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

struct PhoneServiceModule<'lua> {
    name: String,
    phone_number: Option<String>,
    tbl_module: LuaTable<'lua>,
    func_load: Option<LuaFunction<'lua>>,
    func_unload: Option<LuaFunction<'lua>>,
    func_idle_tick: Option<LuaFunction<'lua>>
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
                let func_idle_tick = table.raw_get("idle_tick").unwrap();  

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
                    func_unload, // TODO: Call this in destructor
                    func_idle_tick 
                })
            },
            Err(err) => Err(format!("Unable to load service module: {:#?}", err))
        }
    }

    fn tick(&self) -> LuaResult<()> {
        // TODO: Use call_tick() when appropriate
        if let Some(func_idle_tick) = &self.func_idle_tick {
            func_idle_tick.call::<_, ()>(())?;
        }
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

        // sound.play(path, channel, looping)
        tbl_sound.set("play", lua.create_function(move |_, (path, channel, looping): (String, usize, Option<bool>)| {
            self.sound_engine.borrow().play(path.as_str(), Channel::from(channel), false, looping.unwrap_or(false), true);
            Ok(())
        }).unwrap());

        // sound.play_wait(path, channel, looping)
        tbl_sound.set("play_wait", lua.create_function(move |_, (path, channel, looping): (String, usize, Option<bool>)| {
            self.sound_engine.borrow().play(path.as_str(), Channel::from(channel), true, looping.unwrap_or(false), true);
            Ok(())
        }).unwrap());

        // sound.play_next(path, channel, looping)
        tbl_sound.set("play_next", lua.create_function(move |_, (path, channel, looping): (String, usize, Option<bool>)| {
            self.sound_engine.borrow().play(path.as_str(), Channel::from(channel), true, looping.unwrap_or(false), false);
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