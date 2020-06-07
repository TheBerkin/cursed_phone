mod config;
mod services;
mod phone;
mod sound;
mod gpio;

use crate::services::PbxEngine;
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

fn main() -> Result<(), String> {
    let config = Rc::new(config::load_config(CONFIG_PATH));
    println!("Config loaded: {:#?}", config);
    let tick_interval = time::Duration::from_secs_f64(1.0f64 / config.tick_rate);
    let sound_engine = create_sound_engine(&config);
    let phone = create_phone(&config, sound_engine);
    let pbx = create_pbx(&config, sound_engine);
    pbx.listen_phone_input(phone.gen_phone_output());
    phone.listen_from_pbx(pbx.gen_pbx_output());
    pbx.load_cursed_api()?;
    pbx.load_services();

    loop {
        // Update engine state
        let tick_start = time::Instant::now();
        phone.tick();
        pbx.tick();
        let tick_end = time::Instant::now();

        // Lock tickrate at configured value
        if let Some(delay) = tick_interval.checked_sub(tick_end.saturating_duration_since(tick_start)) {
            thread::sleep(delay);
        }
    }
    Ok(())
}

fn create_pbx<'a>(config: &Rc<CursedConfig>, sound_engine: &Rc<RefCell<SoundEngine>>) -> &'static mut PbxEngine<'a> {
    let pbx = Box::new(PbxEngine::new(SCRIPTS_PATH, config, sound_engine));
    let pbx: &'static mut PbxEngine = Box::leak(pbx);
    pbx
}

fn create_sound_engine(config: &Rc<CursedConfig>) -> &'static mut Rc<RefCell<SoundEngine>> {
    println!("Loading sound engine... ");
    let sound_engine = Box::new(Rc::new(RefCell::new(SoundEngine::new(SOUNDS_PATH, config))));
    let sound_engine: &'static mut Rc<RefCell<SoundEngine>> = Box::leak(sound_engine);
    sound_engine
}

fn create_phone(config: &CursedConfig, sound_engine: &Rc<RefCell<SoundEngine>>) -> &'static mut PhoneEngine {
    println!("Loading phone engine... ");
    let phone_engine = Box::new(PhoneEngine::new(config, sound_engine));
    let phone_engine: &'static mut PhoneEngine = Box::leak(phone_engine);
    phone_engine
}