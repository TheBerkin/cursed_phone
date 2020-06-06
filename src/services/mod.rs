#![allow(dead_code)]

mod props;
mod api;

use std::rc::Rc;
use std::cell::RefCell;
use std::{thread, time, fs};
use std::path::{Path, PathBuf};
use std::time::Instant;
use rand::Rng;
use mlua::prelude::*;
use indexmap::IndexMap;
use crate::sound::*;

pub use self::api::*;
pub use self::props::*;

const BOOTSTRAPPER_SCRIPT_NAME: &str = "bootstrapper";
const API_GLOB: &str = "api/*";

/// A Lua-powered telephone exchange that loads,
/// manages, and runs scripted phone services.
pub struct PbxEngine<'lua> {
    lua: Lua,
    scripts_root: PathBuf,
    start_time: Instant,
    services_by_number: RefCell<IndexMap<String, Rc<ServiceModule<'lua>>>>,
    services_by_name: RefCell<IndexMap<String, Rc<ServiceModule<'lua>>>>,
    sound_engine: Rc<RefCell<SoundEngine>>
}

pub struct ServiceModule<'lua> {
    id: usize,
    name: String,
    phone_number: Option<String>,
    ringback_enabled: bool,
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
                let ringback_enabled: bool = table.raw_get("_ringback_enabled").unwrap_or(true);
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
                    id: 0,
                    ringback_enabled,
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

    fn set_id(&mut self, id: usize) {
        self.id = id;
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn state(&self) -> LuaResult<ServiceState> {
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
impl<'lua> PbxEngine<'lua> {
    pub fn new(scripts_root: impl Into<String>, sound_engine: &Rc<RefCell<SoundEngine>>) -> Self {
        let lua = Lua::new();
        Self {
            lua,
            start_time: Instant::now(),
            scripts_root: Path::new(scripts_root.into().as_str()).canonicalize().unwrap(),
            sound_engine: Rc::clone(sound_engine),
            services_by_number: Default::default(),
            services_by_name: Default::default()
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

    pub fn load_services(&'lua self) {
        self.services_by_number.borrow_mut().clear();
        let search_path = self.scripts_root.join("services").join("**").join("*.lua");
        let search_path_str = search_path.to_str().expect("Failed to create search pattern for service modules");
        let mut next_id: usize = 0;
        for entry in globwalk::glob(search_path_str).expect("Unable to read search pattern for service modules") {
            if let Ok(dir) = entry {
                let module_path = dir.path().canonicalize().expect("Unable to expand service module path");
                let service_module = ServiceModule::from_file(&self.lua, &module_path);
                match service_module {
                    Ok(mut service_module) => {
                        // Apply and increment next ID
                        service_module.set_id(next_id);
                        next_id += 1;

                        let service_module = Rc::new(service_module);

                        // Register service number
                        if let Some(phone_number) = service_module.phone_number.clone() {
                            if !phone_number.is_empty() {
                                self.services_by_number.borrow_mut().insert(phone_number, service_module.clone());
                            }
                        }

                        // Register service name
                        println!("Service loaded: {} (N = {:?}, ID = {})", service_module.name, service_module.phone_number, service_module.id);
                        self.services_by_name.borrow_mut().insert(service_module.name.clone(), service_module);

                    },
                    Err(err) => {
                        println!("Failed to load service module '{:?}': {:#?}", module_path, err);
                    }
                }
            }
        }
    }

    pub fn tick(&self) {
        let service_modules = self.services_by_number.borrow();
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