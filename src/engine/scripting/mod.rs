
use super::*;
use std::{cmp, error::Error, fmt::Display, collections::BTreeSet};
use log::info;
use perlin2d::PerlinNoise2D;
use rand::distributions::Uniform;

mod cron;
mod gpio;
mod phone;
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

        // Override print()
        globals.set("print", lua.create_function(CursedEngine::lua_print)?)?;

        // Global engine functions
        globals.set("rand_int", lua.create_function(Self::lua_rand_int)?)?;
        globals.set("rand_int_i", lua.create_function(Self::lua_rand_int_i)?)?;
        globals.set("rand_int_skip", lua.create_function(Self::lua_rand_int_skip)?)?;
        globals.set("rand_int_normal", lua.create_function(Self::lua_rand_int_normal)?)?;
        globals.set("rand_int_bias_low", lua.create_function(Self::lua_rand_int_bias_low)?)?;
        globals.set("rand_int_bias_high", lua.create_function(Self::lua_rand_int_bias_high)?)?;
        globals.set("rand_int32", lua.create_function(Self::lua_rand_int32)?)?;
        globals.set("rand_float", lua.create_function(Self::lua_rand_float)?)?;
        globals.set("rand_normal", lua.create_function(Self::lua_rand_normal)?)?;
        globals.set("rand_digit", lua.create_function(Self::lua_rand_digit)?)?;
        globals.set("rand_unique_codes", lua.create_function(Self::lua_rand_unique_codes)?)?;
        globals.set("chance", lua.create_function(Self::lua_chance)?)?;
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

        // _caller_dialed_number_impl()
        globals.set("_caller_dialed_number_impl", lua.create_function(move |_, ()| {
            return Ok(self.called_number.borrow().clone())
        })?)?;

        // ====================================================
        // ============= OTHER ENGINE LIBRARIES ===============
        // ====================================================

        self.load_lua_cron_lib()?;
        self.load_lua_gpio_lib()?;
        self.load_lua_phone_lib()?;
        self.load_lua_sound_lib()?;
        self.load_lua_toll_lib()?;

        // Run API scripts
        self.run_scripts_in_path(self.scripts_root.clone())?;
    
        Ok(())
    }

    fn lua_print(lua: &Lua, values: LuaMultiValue) -> LuaResult<()> {
        let mut buffer = String::new();
        let tostring: LuaFunction = lua.globals().raw_get("tostring").unwrap();
        for val in values.iter() {
            if buffer.len() > 0 {
                buffer.push('\t');
            }
            
            let val_str = tostring.call::<LuaValue, String>(val.clone()).unwrap_or(String::from("???"));
            buffer.push_str(val_str.as_str());
        }
        info!("[LUA] {}", buffer);
        Ok(())
    }

    fn lua_perlin(_: &Lua, (x, y, octaves, frequency, persistence, lacunarity, seed): (f64, f64, i32, f64, f64, f64, i32)) -> LuaResult<f64> {
        let perlin = PerlinNoise2D::new(octaves, 1.0, frequency, persistence, lacunarity, (1.0, 1.0), 0.0, seed);
        let noise = perlin.get_noise(x, y);
        Ok(noise)
    }

    fn lua_rand_int(_: &Lua, (min, max): (i64, i64)) -> LuaResult<i64> {
        if min >= max {
            return Ok(min);
        }
        Ok(rand::thread_rng().gen_range(min..max))
    }

    fn lua_rand_int_i(_: &Lua, (min, max): (i64, i64)) -> LuaResult<i64> {
        if min > max {
            return Ok(min);
        }
        Ok(rand::thread_rng().gen_range(min..=max))
    }

    fn lua_rand_digit(_: &Lua, n: Option<usize>) -> LuaResult<String> {
        let n = n.unwrap_or(1);
        let distr = Uniform::new_inclusive::<u32, u32>(0, 9);
        let digits: String = rand::thread_rng().sample_iter(distr).take(n).map(|c| char::from_digit(c, 10).unwrap()).collect();
        Ok(digits)
    }

    fn lua_rand_unique_codes(lua: &Lua, (n, len_min, len_max): (usize, usize, usize)) -> LuaResult<LuaTable> {
        if len_min > len_max {
            lua_error!("rand_unique_codes: min code length cannot be greater than max")
        }
        let distr = Uniform::new_inclusive::<u32, u32>(0, 9);
        let mut set = BTreeSet::new();
        let mut rng = rand::thread_rng();
        for _ in 0..n {
            loop {
                let code_len = rng.gen_range(len_min..=len_max);
                let code_candidate: String = rng.clone().sample_iter(distr).take(code_len).map(|c| char::from_digit(c, 10).unwrap()).collect();
                if set.insert(code_candidate) {
                    break
                }
            }
        }
        lua.create_table_from(set.into_iter().enumerate())
    }

    fn lua_rand_int_bias_low(_: &Lua, (min, max): (i64, i64)) -> LuaResult<i64> {
        if min >= max {
            return Ok(min);
        }
        let mut rng = rand::thread_rng();
        let (a, b) = (rng.gen_range(min..max), rng.gen_range(min..max));
        Ok(cmp::min(a, b))
    }

    fn lua_rand_int_bias_high(_: &Lua, (min, max): (i64, i64)) -> LuaResult<i64> {
        if min >= max {
            return Ok(max);
        }
        let mut rng = rand::thread_rng();
        let (a, b) = (rng.gen_range(min..max), rng.gen_range(min..max));
        Ok(cmp::max(a, b))
    }

    fn lua_rand_int_normal(_: &Lua, (min, max): (i64, i64)) -> LuaResult<i64> {
        if min >= max {
            return Ok(max);
        }
        let mut rng = rand::thread_rng();
        let (a, b) = (rng.gen_range(min..max), rng.gen_range(min..max));
        Ok((a + b) / 2)
    }

    fn lua_rand_int_skip(_: &Lua, (min, skip, max): (i32, i32, i32)) -> LuaResult<i32> {
        if min >= max {
            return Ok(min);
        }
        if skip < min || skip > max {
            Ok(rand::thread_rng().gen_range(min..max))
        } else {
            let range_size: i64 = (max as i64) - (min as i64);
            if range_size > 1 {
                let range_select = rand::thread_rng().gen_range(1..range_size) % range_size;
                let output = min as i64 + range_select;
                Ok(output as i32)
            } else {
                Ok(rand::thread_rng().gen_range(min..max))
            }
        }
    }

    fn lua_rand_int32(_: &Lua, _: ()) -> LuaResult<i32> {
        Ok(rand::thread_rng().gen())
    }

    fn lua_rand_float(_: &Lua, (min, max): (f64, f64)) -> LuaResult<f64> {
        if min >= max {
            return Ok(min);
        }
        Ok(rand::thread_rng().gen_range(min..max))
    }

    fn lua_rand_normal(_: &Lua, (min, max): (f64, f64)) -> LuaResult<f64> {
        if min >= max {
            return Ok(min)
        }
        let mut rng = rand::thread_rng();
        let (a, b) = (rng.gen_range(min..max), rng.gen_range(min..max));
        Ok((a + b) / 2.0)
    }

    fn lua_chance(_: &Lua, p: f64) -> LuaResult<bool> {
        match p {
            p if {p < 0.0 || p.is_nan()} => Ok(false),
            p if {p > 1.0} => Ok(true),
            p => Ok(rand::thread_rng().gen_bool(p))
        }
    }
}