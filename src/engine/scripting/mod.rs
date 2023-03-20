use crate::engine::scripting::random::LuaRandom;

use super::*;
use std::{error::Error, fmt::Display};
use log;
use perlin2d::PerlinNoise2D;
use rand::RngCore;

mod cron;
mod gpio;
mod phone;
mod logging;
mod random;
mod sound;
mod toll;

#[derive(Debug)]
pub(self) struct CustomLuaError {
    message: String
}

impl CustomLuaError {
    pub fn new(message: String) -> Self {
        Self {
            message
        }
    }
}

impl Error for CustomLuaError {}

impl Display for CustomLuaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

macro_rules! lua_error {
    ($($arg:tt)*) => {
        return Err(LuaError::ExternalError(Arc::new(crate::engine::scripting::CustomLuaError::new(format!($($arg)*)))))
    }
}
pub(self) use lua_error;

#[allow(unused_must_use)]
impl<'lua> CursedEngine<'lua> {    
    pub fn load_lua_api(&'static self) -> LuaResult<()> {
        info!("Setting up Lua...");
    
        let lua = &self.lua;

        let globals = &lua.globals();

        globals.set("DEVMODE", cfg!(feature = "devmode"));

        globals.set("newrng", lua.create_function(move |_, seed: Option<u64>| {
            Ok(LuaRandom::with_seed(seed.unwrap_or_else(|| rand::thread_rng().next_u64())))
        })?)?;

        // Override print()
        globals.set("print", lua.create_function(move |lua, values: LuaMultiValue| {
            Self::lua_log_print(lua, values, log::Level::Info, 0);
            Ok(())
        })?)?;

        // Global engine functions
        globals.set("perlin_sample", lua.create_function(Self::lua_perlin)?)?;

        globals.set("engine_time", lua.create_function(move |_, ()| {
            let run_time = self.start_time.elapsed().as_secs_f64();
            Ok(run_time)
        })?)?;

        globals.set("call_time", lua.create_function(move |_, ()| {
            match self.state() {
                PhoneLineState::Connected => {
                    return Ok(self.current_state_time().as_secs_f64());
                }
                _ => Ok(0.0)
            }
        })?)?;

        // set_agent_sounds_loaded(agent_id, loaded)
        globals.set("set_agent_sounds_loaded", lua.create_function(move |_, (agent_id, load): (AgentId, bool)| {
            let sound_engine = &self.sound_engine;
            if let Some(agent) = self.lookup_agent_id(agent_id) {
                if load {
                    agent.load_sound_banks(sound_engine)
                } else {
                    agent.unload_sound_banks(sound_engine)
                }
            }
            Ok(())
        })?)?;

        // ====================================================
        // ============= OTHER ENGINE LIBRARIES ===============
        // ====================================================

        self.load_lua_cron_lib()?;
        self.load_lua_gpio_lib()?;
        self.load_lua_phone_lib()?;
        self.load_lua_sound_lib()?;
        self.load_lua_toll_lib()?;
        self.load_lua_log_lib()?;

        // Run API scripts
        self.run_scripts_in_path(self.scripts_root.clone())?;
    
        Ok(())
    }

    fn lua_log_print(lua: &Lua, values: LuaMultiValue, level: log::Level, stack_level: usize) -> LuaResult<()> {
        let mut buffer = String::new();
        
        if let Some(debug_info) = lua.inspect_stack(stack_level.saturating_add(1)) {
            let src_name = debug_info.source().source.map(String::from_utf8_lossy).unwrap_or_default();
            let msg = format!("[{}:{}] ", src_name, debug_info.curr_line());
            buffer.push_str(msg.as_str());
        }

        for (i, val) in values.iter().enumerate() {
            if i > 0 {
                buffer.push('\t');
            }
            
            if let Some(val_str) = lua.coerce_string(val.clone())? {
                if let Ok(s) = val_str.to_str() {
                    buffer.push_str(s);
                }
            }
        }
        log::log!(level, "{}", buffer);
        Ok(())
    }

    fn lua_perlin(_: &Lua, (x, y, octaves, frequency, persistence, lacunarity, seed): (f64, f64, i32, f64, f64, f64, i32)) -> LuaResult<f64> {
        let perlin = PerlinNoise2D::new(octaves, 1.0, frequency, persistence, lacunarity, (1.0, 1.0), 0.0, seed);
        let noise = perlin.get_noise(x, y);
        Ok(noise)
    }
}