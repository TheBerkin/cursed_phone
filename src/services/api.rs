
use super::*;

#[allow(unused_must_use)]
impl<'lua> PbxEngine<'lua> {    
    pub fn load_cursed_api(&'static self) -> Result<(), String> {
        // Run bootstrapper script
        println!("Bootstrapping Lua...");
        self.run_script(BOOTSTRAPPER_SCRIPT_NAME)?;
    
        let lua = &self.lua;
        let globals = &lua.globals();
        let tbl_sound = lua.create_table().unwrap();
        let tbl_service = lua.create_table().unwrap();

        // ====================================================
        // ================ MISC API FUNCTIONS ================
        // ====================================================
    
        // sleep()
        globals.set("sleep", lua.create_function(PbxEngine::lua_sleep).unwrap());
        // random_int()
        globals.set("random_int", lua.create_function(PbxEngine::lua_random_int).unwrap());
        // random_float()
        globals.set("random_float", lua.create_function(PbxEngine::lua_random_float).unwrap());
    
        // get_run_time()
        globals.set("get_run_time", lua.create_function(move |_, ()| {
            let run_time = self.start_time.elapsed().as_secs_f64();
            Ok(run_time)
        }).unwrap());
    
        // ====================================================
        // ==================== SOUND API =====================
        // ====================================================
    
        // sound.play(path, channel, opts)
        tbl_sound.set("play", lua.create_function(move |_, (path, channel, opts): (String, usize, Option<LuaTable>)| {
            let mut speed: Option<f32> = None;
            let mut interrupt: Option<bool> = None;
            let mut looping: Option<bool> = None;
            let mut volume: Option<f32> = None;
            if let Some(opts_table) = opts {
                speed = opts_table.get::<&str, f32>("speed").ok();
                interrupt = opts_table.get::<&str, bool>("interrupt").ok();
                looping = opts_table.get::<&str, bool>("looping").ok();
                volume = opts_table.get::<&str, f32>("volume").ok();
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
    
        // sound.play_fast_busy_tone()
        tbl_sound.set("play_fast_busy_tone", lua.create_function(move |_, ()| {
            self.sound_engine.borrow().play_fast_busy_tone();
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
    
        // Run API scripts
        self.run_scripts_in_glob(API_GLOB)?;
    
        Ok(())
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
}