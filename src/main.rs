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
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{thread, time};
use log::{info, warn};
use simplelog::{TermLogger, LevelFilter, TerminalMode, ColorChoice};
use thread_priority::*;
use ctrlc;
use vfs::{OverlayFS, VfsPath, AltrootFS, PhysicalFS};

const CONFIG_PATH: &str = "./cursed_phone.conf";
const VFS_SCRIPTS_PATH: &str = "./scripts";
const VFS_SOUNDS_PATH: &str = "./sounds";
const VFS_SOUNDBANKS_PATH: &str = "./soundbanks";

const ENV_CONFIG_PATH: &str = "CURSED_CONFIG_PATH";

#[allow(unreachable_code)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    let vfs_root = create_virtual_filesystem();
    let sound_engine = create_sound_engine(&config, &vfs_root);
    let phone = create_phone(&config, sound_engine);
    let engine = create_cursed_engine(&config, sound_engine, &vfs_root);
    engine.listen(phone.gen_phone_output());
    phone.listen(engine.gen_engine_output());
    engine.load_lua_api()?;
    engine.load_agents();

    let is_running = Arc::new(AtomicBool::new(true));
    let is_running_c = Arc::clone(&is_running);

    ctrlc::set_handler(move || {
        info!("Ctrl+C detected; shutting down.");
        is_running_c.store(false, Ordering::SeqCst);
    }).expect("unable to set ctrl-c handler");

    info!("Phone ready.");

    while is_running.load(Ordering::SeqCst) {
        // Update engine state
        let tick_start = time::Instant::now();
        phone.tick();
        engine.tick();
        let tick_end = time::Instant::now();

        // Lock tickrate at configured value
        if let Some(delay) = tick_interval.checked_sub(tick_end.saturating_duration_since(tick_start)) {
            thread::sleep(delay);
        }
    }
    Ok(())
}

fn create_virtual_filesystem() -> VfsPath {
    let vfs_static = AltrootFS::new(PhysicalFS::new("./res").into());
    let mut resource_paths: Vec<VfsPath> = vec![];
    resource_paths.push(vfs_static.into());
    if let Ok(walker) = globwalk::glob(env::current_dir().unwrap().join("res/addons/*/").to_string_lossy()) {
        for addon_dir in walker {
            if let Ok(entry) = addon_dir {
                if entry.file_type().is_dir() {
                    info!("Mounting addon resources: {}", entry.file_name().to_string_lossy());
                    let addon_vfs = AltrootFS::new(PhysicalFS::new(entry.path()).into());
                    resource_paths.push(addon_vfs.into());
                }
            }
        }
    }
    let vfs = OverlayFS::new(&resource_paths);
    return vfs.into()
}

fn create_cursed_engine<'a>(config: &Rc<CursedConfig>, sound_engine: &Rc<RefCell<SoundEngine>>, vfs_root: &VfsPath) -> &'static mut CursedEngine<'a> {
    let pbx = Box::new(CursedEngine::new(vfs_root.join(VFS_SCRIPTS_PATH).unwrap(), config, sound_engine));
    let pbx: &'static mut CursedEngine = Box::leak(pbx);
    pbx
}

fn create_sound_engine(config: &Rc<CursedConfig>, vfs_root: &VfsPath) -> &'static mut Rc<RefCell<SoundEngine>> {
    info!("Loading sound engine... ");
    let sound_engine = Box::new(Rc::new(RefCell::new(
        SoundEngine::new(vfs_root.join(VFS_SOUNDS_PATH).unwrap(), vfs_root.join(VFS_SOUNDBANKS_PATH).unwrap(), config))));
    let sound_engine: &'static mut Rc<RefCell<SoundEngine>> = Box::leak(sound_engine);
    sound_engine
}

fn create_phone(config: &Rc<CursedConfig>, sound_engine: &Rc<RefCell<SoundEngine>>) -> &'static mut PhoneEngine {
    info!("Loading phone engine... ");
    let phone_engine = Box::new(PhoneEngine::new(config, sound_engine));
    let phone_engine: &'static mut PhoneEngine = Box::leak(phone_engine);
    phone_engine
}