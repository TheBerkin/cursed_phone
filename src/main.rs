mod lua;
mod phone;
mod sound;

use crate::lua::LuaEngine;
use crate::sound::SoundEngine;
use crate::phone::PhoneEngine;
use serde::Deserialize;
use serde_json;
use std::boxed::Box;
use std::fs::File;
use std::io::BufReader;
use std::rc::Rc;
use std::cell::RefCell;
use std::{thread, time};

const SCRIPTS_PATH: &str = "./res/scripts";
const CONFIG_PATH: &str = "./res/cursed_config.json";
const SOUNDS_PATH: &str = "./res/sounds";
const TICK_RATE_MS: u64 = 30;

#[derive(Deserialize, Copy, Clone, Debug)]
pub struct GpioPinsConfig {
    in_dial_pulse: u8,
    in_dial_switch: u8,
    in_hook: u8,
    in_motion: u8,
    out_ringer: u8,
    out_vibrate: u8,
}

#[derive(Deserialize, Clone, Debug)]
pub struct CursedConfig {
    phone_type: String,
    pdd: f32,
    volume: f32,
    off_hook_delay: f32,
    gpio_pins: GpioPinsConfig,
    enable_ringer: bool,
    enable_vibration: bool,
    enable_motion_sensor: bool
}

fn main() -> Result<(), String> {
    let config = load_config();
    println!("Config loaded: {:#?}", config);
    let sound_engine = create_sound_engine(&config);
    let phone_engine = create_phone_engine(&config);
    let lua_engine = create_lua_engine(sound_engine);
    lua_engine.load_cursed_api()?;
    lua_engine.load_services();

    sound_engine.borrow().play_off_hook_tone(0.25);

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
    let sound_engine = Box::new(Rc::new(RefCell::new(SoundEngine::new(SOUNDS_PATH, config.volume))));
    let sound_engine: &'static mut Rc<RefCell<SoundEngine>> = Box::leak(sound_engine);
    sound_engine
}

fn create_phone_engine(config: &CursedConfig) -> &'static mut Rc<RefCell<PhoneEngine>> {
    let phone_engine = Box::new(Rc::new(RefCell::new(PhoneEngine::new(config))));
    let phone_engine: &'static mut Rc<RefCell<PhoneEngine>> = Box::leak(phone_engine);
    phone_engine
}

fn load_config() -> CursedConfig {
    let file = File::open(CONFIG_PATH).expect("Unable to open config file");
    let reader = BufReader::new(file);
    let config: CursedConfig =
        serde_json::from_reader(reader).expect("Unable to parse config file");
    config
}