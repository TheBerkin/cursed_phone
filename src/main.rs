mod config;
mod engine;
mod phone;
mod sound;
mod gpio;

use crate::engine::CursedEngine;
use crate::sound::SoundEngine;
use crate::phone::PhoneEngine;
use crate::config::*;
use std::boxed::Box;
use std::rc::Rc;
use std::env;
use std::cell::RefCell;
use std::{thread, time};
use log::{info, warn};
use simplelog::{TermLogger, LevelFilter, TerminalMode, ColorChoice};
use thread_priority::*;

const SCRIPTS_PATH: &str = "./res/scripts";
const CONFIG_PATH: &str = "./cursed_phone.conf";
const SOUNDS_PATH: &str = "./res/sounds";
const SOUNDBANKS_PATH: &str = "./res/soundbanks";

const ENV_CONFIG_PATH: &str = "CURSED_CONFIG_PATH";

#[allow(unreachable_code)]
fn main() -> Result<(), String> {
    // Set up logger
    TermLogger::init(LevelFilter::Info, Default::default(), TerminalMode::Mixed, ColorChoice::Auto).unwrap();

    // Set thread priority
    if let Err(err) = set_current_thread_priority(ThreadPriority::Max) {
        warn!("Failed to raise thread priority: {:?}", err);
    }

    // Load engine
    let env_config_path = env::var(ENV_CONFIG_PATH);
    let config_path = env_config_path.as_deref().unwrap_or(CONFIG_PATH);
    info!("Loading config: {}", config_path);
    let config = Rc::new(config::load_config(config_path));
    info!("Config loaded: {:#?}", config);
    let tick_interval = time::Duration::from_secs_f64(1.0f64 / config.tick_rate);
    let sound_engine = create_sound_engine(&config);
    let phone = create_phone(&config, sound_engine);
    let pbx = create_cursed_engine(&config, sound_engine);
    pbx.listen_phone_input(phone.gen_phone_output());
    phone.listen_from_pbx(pbx.gen_pbx_output());
    pbx.load_cursed_lua_api()?;
    pbx.load_agents();

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

fn create_cursed_engine<'a>(config: &Rc<CursedConfig>, sound_engine: &Rc<RefCell<SoundEngine>>) -> &'static mut CursedEngine<'a> {
    let pbx = Box::new(CursedEngine::new(SCRIPTS_PATH, config, sound_engine));
    let pbx: &'static mut CursedEngine = Box::leak(pbx);
    pbx
}

fn create_sound_engine(config: &Rc<CursedConfig>) -> &'static mut Rc<RefCell<SoundEngine>> {
    info!("Loading sound engine... ");
    let sound_engine = Box::new(Rc::new(RefCell::new(
        SoundEngine::new(SOUNDS_PATH, SOUNDBANKS_PATH, config))));
    let sound_engine: &'static mut Rc<RefCell<SoundEngine>> = Box::leak(sound_engine);
    sound_engine
}

fn create_phone(config: &Rc<CursedConfig>, sound_engine: &Rc<RefCell<SoundEngine>>) -> &'static mut PhoneEngine {
    info!("Loading phone engine... ");
    let phone_engine = Box::new(PhoneEngine::new(config, sound_engine));
    let phone_engine: &'static mut PhoneEngine = Box::leak(phone_engine);
    phone_engine
}