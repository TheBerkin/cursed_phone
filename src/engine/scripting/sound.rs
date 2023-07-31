use crate::engine::*;
use super::lua_error;

impl<'lua> CursedEngine<'lua> {    
    pub(super) fn load_lua_sound_lib(&'static self) -> LuaResult<()> { 
        let lua = &self.lua;
        let globals = &lua.globals();

        let tbl_sound = lua.create_table()?;
    
        // sound.play(path, channel, opts)
        tbl_sound.set("play", lua.create_function(move |_, (path, channel, opts): (String, usize, Option<LuaTable>)| {
            let mut speed: Option<f32> = None;
            let mut interrupt: Option<bool> = None;
            let mut looping: Option<bool> = None;
            let mut volume: Option<f32> = None;
            let mut skip: Option<SoundPlaySkip> = None;
            let mut take: Option<Duration> = None;
            let mut delay: Option<Duration> = None;
            let mut fadein: Option<Duration> = None;
            if let Some(opts_table) = opts {
                speed = opts_table.get::<&str, f32>("speed").ok();
                interrupt = opts_table.get::<&str, bool>("interrupt").ok();
                looping = opts_table.get::<&str, bool>("looping").ok();
                volume = opts_table.get::<&str, f32>("volume").ok();
                skip = opts_table.get::<&str, SoundPlaySkip>("skip").ok();
                take = opts_table.get::<&str, f32>("take").ok().map(|secs| Duration::from_secs_f32(secs));
                delay = opts_table.get::<&str, f32>("delay").ok().map(|secs| Duration::from_secs_f32(secs));
                fadein = opts_table.get::<&str, f32>("fadein").ok().map(|secs| Duration::from_secs_f32(secs));
            }
            let info = self.sound_engine.borrow().play(
                path.as_str(), 
                Channel::from(channel), 
                false, 
                interrupt.unwrap_or(true),
                SoundPlayOptions {
                    looping: looping.unwrap_or(false),
                    speed: speed.unwrap_or(1.0),
                    volume: volume.unwrap_or(1.0),
                    skip: skip.unwrap_or_default(),
                    take,
                    delay,
                    fadein: fadein.unwrap_or_default(),
                }
            );

            Ok(match info {
                Some(info) => (true, info.duration.map(|d| {
                    if let Some(take) = take {
                        if take < d {
                            return take.as_secs_f64()
                        }
                    }
                    d.as_secs_f64()
                })),
                None => (false, None)
            })
        })?)?;
    
        // sound.is_busy(channel)
        tbl_sound.set("is_busy", lua.create_function(move |_, channel: usize| {
            let busy = self.sound_engine.borrow().channel_busy(Channel::from(channel));
            Ok(busy)
        })?)?;
    
        // sound.stop(channel)
        tbl_sound.set("stop", lua.create_function(move |_, channel: usize| {
            self.sound_engine.borrow().stop(Channel::from(channel));
            Ok(())
        })?)?;
    
        // sound.stop_all()
        tbl_sound.set("stop_all", lua.create_function(move |_, ()| {
            self.sound_engine.borrow().stop_all();
            Ok(())
        })?)?;
    
        // sound.get_channel_volume(channel)
        tbl_sound.set("get_channel_volume", lua.create_function(move |_, channel: usize| {
            let vol = self.sound_engine.borrow().volume(Channel::from(channel));
            Ok(vol)
        })?)?;
    
        // sound.set_channel_volume(channel, volume)
        tbl_sound.set("set_channel_volume", lua.create_function(move |_, (channel, volume): (usize, f32)| {
            self.sound_engine.borrow_mut().set_volume(Channel::from(channel), volume);
            Ok(())
        })?)?;

        // sound.get_master_volume()
        tbl_sound.set("get_master_volume", lua.create_function(move |_, ()| {
            Ok(self.sound_engine.borrow_mut().master_volume())
        })?)?;

        // sound.set_master_volume(volume)
        tbl_sound.set("set_master_volume", lua.create_function(move |_, volume: f32| {
            self.sound_engine.borrow_mut().set_master_volume(volume);
            Ok(())
        })?)?;

        // sound.get_channel_fade_volume(channel)
        tbl_sound.set("get_channel_fade_volume", lua.create_function(move |_, channel: usize| {
            let vol = self.sound_engine.borrow().fade_volume(Channel::from(channel));
            Ok(vol)
        })?)?;
    
        // sound.set_channel_fade_volume(channel, volume)
        tbl_sound.set("set_channel_fade_volume", lua.create_function(move |_, (channel, volume): (usize, f32)| {
            self.sound_engine.borrow_mut().set_fade_volume(Channel::from(channel), volume);
            Ok(())
        })?)?;

        // sound.is_channel_muted(channel)
        tbl_sound.set("is_channel_muted", lua.create_function(move |_, channel: usize| {
            Ok(self.sound_engine.borrow().is_muted(Channel::from(channel)))
        })?)?;

        // sound.set_channel_muted(channel, muted)
        tbl_sound.set("set_channel_muted", lua.create_function(move |_, (channel, muted): (usize, bool)| {
            self.sound_engine.borrow_mut().set_muted(Channel::from(channel), muted);
            Ok(())
        })?)?;
    
        // sound.play_dial_tone()
        tbl_sound.set("play_dial_tone", lua.create_function(move |_, ()| {
            self.sound_engine.borrow().play_dial_tone();
            Ok(())
        })?)?;
    
        // sound.play_busy_tone()
        tbl_sound.set("play_busy_tone", lua.create_function(move |_, ()| {
            self.sound_engine.borrow().play_busy_tone();
            Ok(())
        })?)?;
    
        // sound.play_fast_busy_tone()
        tbl_sound.set("play_fast_busy_tone", lua.create_function(move |_, ()| {
            self.sound_engine.borrow().play_fast_busy_tone();
            Ok(())
        })?)?;
    
        // sound.play_ringback_tone()
        tbl_sound.set("play_ringback_tone", lua.create_function(move |_, ()| {
            self.sound_engine.borrow().play_ringback_tone();
            Ok(())
        })?)?;

        // sound.play_off_hook_tone()
        tbl_sound.set("play_off_hook_tone", lua.create_function(move |_, ()| {
            self.sound_engine.borrow().play_off_hook_tone();
            Ok(())
        })?)?;

        // sound.play_special_info_tone(sit_type)
        tbl_sound.set("play_special_info_tone", lua.create_function(move |_, sit_type: u8| {
            let sit = SpecialInfoTone::from(sit_type);
            self.sound_engine.borrow().play_special_info_tone(sit);
            Ok(())
        })?)?;
    
        // sound.play_dtmf_digit(digit, duration, volume)
        tbl_sound.set("play_dtmf_digit", lua.create_function(move |_, (digit_str, duration, volume): (String, f64, f32)| {
            if let Some(digit) = digit_str.chars().next() {
                self.sound_engine.borrow().play_dtmf(digit, Duration::from_secs_f64(duration), volume);
            } else {
                lua_error!("digit string is empty");
            }
            Ok(())
        })?)?;
    
        globals.set("sound", tbl_sound)?;

        Ok(())
    }
}