mod config;
mod lua;
mod phone;
mod sound;
mod gpio;

use crate::lua::LuaEngine;
use crate::sound::SoundEngine;
use crate::phone::PhoneEngine;
use crate::config::*;
use std::boxed::Box;
use std::rc::Rc;
use std::cell::RefCell;
use std::{thread, time};

const SCRIPTS_PATH: &str = "./res/scripts";
const CONFIG_PATH: &str = "./cursed_phone.conf";
const SOUNDS_PATH: &str = "./res/sounds";
const TICK_RATE_MS: u64 = 30;

fn main() -> Result<(), String> {
    let config = config::load_config(CONFIG_PATH);
    println!("Config loaded: {:#?}", config);
    let sound_engine = create_sound_engine(&config);
    let phone_engine = create_phone_engine(&config);
    let lua_engine = create_lua_engine(sound_engine);
    lua_engine.load_cursed_api()?;
    lua_engine.load_services();

    loop {
        lua_engine.tick();
        thread::sleep(time::Duration::from_millis(TICK_RATE_MS));
    }
    Ok(())
}

fn create_lua_engine(sound_engine: &Rc<RefCell<SoundEngine>>) -> &'static mut LuaEngine {
    let lua_engine = Box::new(LuaEngine::new(SCRIPTS_PATH, sound_engine));
    let lua_engine: &'static mut LuaEngine = Box::leak(lua_engine);
    lua_engine
}

fn create_sound_engine(config: &CursedConfig) -> &'static mut Rc<RefCell<SoundEngine>> {
    let sound_engine = Box::new(Rc::new(RefCell::new(SoundEngine::new(SOUNDS_PATH, config.sound))));
    let sound_engine: &'static mut Rc<RefCell<SoundEngine>> = Box::leak(sound_engine);
    sound_engine
}

fn create_phone_engine(config: &CursedConfig) -> &'static mut Rc<RefCell<PhoneEngine>> {
    let phone_engine = Box::new(Rc::new(RefCell::new(PhoneEngine::new(config))));
    let phone_engine: &'static mut Rc<RefCell<PhoneEngine>> = Box::leak(phone_engine);
    phone_engine
}