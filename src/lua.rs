#![allow(dead_code)]

use crate::sound::*;
use std::sync::Arc;
use std::cell::RefCell;
use std::{thread, time};
use mlua::prelude::*;
use ref_portals::sync::RwAnchor;

pub struct LuaEngine<'phone> {
    lua: Lua,
    sound_engine_anchor: RwAnchor<'phone, SoundEngine<'phone>>
}

#[allow(unused_must_use)]
impl<'phone> LuaEngine<'phone> {
    pub fn new(sound_engine: &'phone mut SoundEngine<'phone>) -> Self {
        let lua = Lua::new();
        Self {
            lua,
            sound_engine_anchor: RwAnchor::new(sound_engine)
        }
    }

    fn lua_sleep(_: &Lua, ms: u64) -> LuaResult<()> {
        thread::sleep(time::Duration::from_millis(ms));
        Ok(())
    }

    pub fn create_phone_api(&mut self) {
        let lua = &self.lua;
        let globals = &lua.globals();
        let sound_engine_portal = self.sound_engine_anchor.portal();

        // sleep()
        globals.set("sleep", lua.create_function(LuaEngine::lua_sleep).unwrap());

        // test_sound()
        globals.set("test_sound", lua.create_function(move |_, ()| {
            sound_engine_portal.read().play_annoying_sine(Channel::Bg1, 1000);
            Ok(())
        }).unwrap());
    }

    pub fn test(&self) {
        let src = self.lua.load("test_sound()");
        src.exec();
    }

    pub fn tick(&self) {
    }
}