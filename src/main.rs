mod lua;
mod phone;
mod sound;

use crate::lua::LuaEngine;
use crate::sound::SoundEngine;
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
    in_dial_pulse: i32,
    in_dial_switch: i32,
    in_hook: i32,
    in_motion: i32,
    out_ringer: i32,
    out_vibrate: i32,
}

#[derive(Deserialize, Copy, Clone, Debug)]
pub struct CursedConfig {
    pdd: f32,
    volume: f32,
    off_hook_delay: f32,
    gpio_pins: GpioPinsConfig,
}

fn main() -> Result<(), String> {
    let config = load_config();
    println!("Config loaded: {:#?}", config);
    let sound_engine = create_sound_engine();
    let lua_engine = create_lua_engine(sound_engine);
    lua_engine.load_cursed_api()?;
    lua_engine.load_services();

    loop {
        lua_engine.tick();
        sleep(TICK_RATE_MS);
    }

    cleanup_sound_engine(sound_engine);
}

fn create_lua_engine(sound_engine: &'static Rc<RefCell<SoundEngine>>) -> &'static mut LuaEngine {
    let lua_engine = Box::new(LuaEngine::new(SCRIPTS_PATH, sound_engine));
    let lua_engine: &'static mut LuaEngine = Box::leak(lua_engine);
    lua_engine
}

fn create_sound_engine() -> &'static mut Rc<RefCell<SoundEngine>> {
    let sound_engine = Box::new(Rc::new(RefCell::new(SoundEngine::new(SOUNDS_PATH))));
    let sound_engine: &'static mut Rc<RefCell<SoundEngine>> = Box::leak(sound_engine);
    sound_engine
}

fn cleanup_sound_engine(sound_engine: &'static mut Rc<RefCell<SoundEngine>>) {
    unsafe { Box::from_raw(sound_engine) };
}

fn load_config() -> CursedConfig {
    let file = File::open(CONFIG_PATH).expect("Unable to open config file");
    let reader = BufReader::new(file);
    let config: CursedConfig =
        serde_json::from_reader(reader).expect("Unable to parse config file");
    config
}

fn sleep(ms: u64) {
    thread::sleep(time::Duration::from_millis(ms));
}
