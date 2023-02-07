
use super::*;
use std::{cmp};
use log::{info};

mod cron;
mod gpio;
mod phone;
mod sound;
mod toll;

#[allow(unused_must_use)]
impl<'lua> CursedEngine<'lua> {    
    pub fn load_lua_api(&'static self) -> LuaResult<()> {
        info!("Setting up Lua...");
    
        let lua = &self.lua;
        let globals = &lua.globals();

        // Override print()
        globals.set("print", lua.create_function(CursedEngine::lua_print)?)?;

        // Run setup script
        self.run_script(SETUP_SCRIPT_NAME)?;

        // ====================================================
        // ============== GENERAL API FUNCTIONS ===============
        // ====================================================
    
        // sleep()
        globals.set("sleep", lua.create_function(CursedEngine::lua_sleep)?)?;

        // rand_int(min, max)
        globals.set("rand_int", lua.create_function(CursedEngine::lua_rand_int)?)?;

        // rand_int_skip(min, skip, max)
        globals.set("rand_int_skip", lua.create_function(CursedEngine::lua_rand_int_skip)?)?;

        // rand_int_bias_low(min, max)
        globals.set("rand_int_bias_low", lua.create_function(CursedEngine::lua_rand_int_bias_low)?)?;

        // rand_int_bias_high(min, max)
        globals.set("rand_int_bias_high", lua.create_function(CursedEngine::lua_rand_int_bias_high)?)?;

        // rand_float(min, max)
        globals.set("rand_float", lua.create_function(CursedEngine::lua_rand_float)?)?;

        // chance(p)
        globals.set("chance", lua.create_function(CursedEngine::lua_chance)?)?;

        // engine_time()
        globals.set("engine_time", lua.create_function(move |_, ()| {
            let run_time = self.start_time.elapsed().as_secs_f64();
            Ok(run_time)
        })?)?;

        // call_time()
        globals.set("call_time", lua.create_function(move |_, ()| {
            match self.state() {
                PhoneLineState::Connected => {
                    return Ok(self.current_state_time().as_secs_f64());
                }
                _ => Ok(0.0)
            }
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
        self.run_scripts_in_glob(API_GLOB)?;
    
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

    fn lua_sleep(_: &Lua, ms: u64) -> LuaResult<()> {
        thread::sleep(time::Duration::from_millis(ms));
        Ok(())
    }

    fn lua_rand_int(_: &Lua, (min, max): (i32, i32)) -> LuaResult<i32> {
        if min >= max {
            return Ok(min);
        }
        Ok(rand::thread_rng().gen_range(min..max))
    }

    fn lua_rand_int_bias_low(_: &Lua, (min, max): (i32, i32)) -> LuaResult<i32> {
        if min >= max {
            return Ok(min);
        }
        let mut rng = rand::thread_rng();
        let (a, b) = (rng.gen_range(min..max), rng.gen_range(min..max));
        Ok(cmp::min(a, b))
    }

    fn lua_rand_int_bias_high(_: &Lua, (min, max): (i32, i32)) -> LuaResult<i32> {
        if min >= max {
            return Ok(max);
        }
        let mut rng = rand::thread_rng();
        let (a, b) = (rng.gen_range(min..max), rng.gen_range(min..max));
        Ok(cmp::max(a, b))
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

    fn lua_rand_float(_: &Lua, (min, max): (f64, f64)) -> LuaResult<f64> {
        if min >= max {
            return Ok(min);
        }
        Ok(rand::thread_rng().gen_range(min..max))
    }

    fn lua_chance(_: &Lua, p: f64) -> LuaResult<bool> {
        match p {
            p if {p < 0.0 || p.is_nan()} => Ok(false),
            p if {p > 1.0} => Ok(true),
            p => Ok(rand::thread_rng().gen_bool(p))
        }
    }
}