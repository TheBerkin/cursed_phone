mod phone;
mod lua;
mod sound;

use std::fs::File;
use std::io::BufReader;
use serde::Deserialize;
use serde_json;
use std::{thread, time};
use crate::sound::{SoundEngine, Channel};
use crate::lua::LuaEngine;

const CONFIG_PATH: &str = "./res/cursed_config.json";
const SOUNDS_PATH: &str = "./res/sounds";

#[derive(Deserialize, Copy, Clone, Debug)]
pub struct GpioPinsConfig {
    in_dial_pulse: i32,
    in_dial_switch: i32,
    in_hook: i32,
    in_motion: i32,
    out_ringer: i32,
    out_vibrate: i32
}

#[derive(Deserialize, Copy, Clone, Debug)]
pub struct CursedConfig {
    pdd: f32,
    volume: f32,
    off_hook_delay: f32,
    gpio_pins: GpioPinsConfig
}

fn main() {
    let config = load_config();
    println!("Config loaded: {:#?}", config);
    let sound_engine = SoundEngine::new(SOUNDS_PATH);
    let mut lua_engine = LuaEngine::new(&mut &sound_engine);
    lua_engine.create_phone_api();

    lua_engine.test();

    loop {
        lua_engine.tick();
        sleep(1000);
    }
}

fn load_config() -> CursedConfig {
    let file = File::open(CONFIG_PATH).expect("Unable to open config file");
    let reader = BufReader::new(file);
    let config: CursedConfig = serde_json::from_reader(reader).expect("Unable to parse config file");
    config
}

fn sleep(ms: u64) {
    thread::sleep(time::Duration::from_millis(ms));
}