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
const VFS_AGENTS_PATH: &str = "./agents";
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
    let vfs_root = create_virtual_filesystem(&config);
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

    let tick_interval = time::Duration::from_secs_f64(1.0f64 / config.tick_rate);
    let mut fps = 0;
    let mut frame_count = 0;
    let mut last_fps_check = time::Instant::now();

    while is_running.load(Ordering::SeqCst) {
        // Update engine state
        let tick_start = time::Instant::now();
        phone.tick();
        engine.tick();
        let tick_end = time::Instant::now();

        frame_count += 1;
        if tick_end.saturating_duration_since(last_fps_check).as_secs() >= 1 {
            last_fps_check = tick_end;
            fps = frame_count;
            frame_count = 0;
            info!("fps = {}", fps);
        }

        // Lock tickrate at configured value
        let frame_time = tick_end.saturating_duration_since(tick_start);
        let remaining_tick_time = tick_interval.saturating_sub(frame_time);
        spin_sleep::sleep(remaining_tick_time);
    }
    Ok(())
}

fn create_virtual_filesystem(config: &CursedConfig) -> VfsPath {
    let mut resource_paths: Vec<VfsPath> = vec![];
    for pattern in config.include_resources.iter() {
        if let Ok(walker) = glob::glob(pattern.as_str()) {
            for entry in walker {
                if let Ok(entry) = entry {
                    if !entry.is_dir() { continue }
                    info!("Mounting resources: {}", entry.to_string_lossy());
                    let addon_vfs = AltrootFS::new(PhysicalFS::new(entry).into());
                    resource_paths.push(addon_vfs.into());
                }
            }
        }
    }
    let vfs = OverlayFS::new(&resource_paths);
    return vfs.into()
}

fn create_cursed_engine<'a>(config: &Rc<CursedConfig>, sound_engine: &Rc<RefCell<SoundEngine>>, vfs_root: &VfsPath) -> &'static mut CursedEngine<'a> {
    let scripts_root = vfs_root.join(VFS_SCRIPTS_PATH).unwrap();
    let agents_root = vfs_root.join(VFS_AGENTS_PATH).unwrap();
    let engine = Box::new(CursedEngine::new(scripts_root, agents_root, config, sound_engine));
    let engine: &'static mut CursedEngine = Box::leak(engine);
    engine
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