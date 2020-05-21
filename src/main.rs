mod sound;

use std::{thread, time};
use crate::sound::{SoundEngine, Channel};

const SOUNDS_PATH: &str = "./res/sounds";

fn main() {
    let sound_engine = SoundEngine::new(SOUNDS_PATH);

    loop {
        sound_engine.play_wait("denise/thinking/*", Channel::Tone);
        sleep(1000);
    }
}

fn sleep(ms: u64) {
    thread::sleep(time::Duration::from_millis(ms));
}