mod sound;

use std::{thread, time};
use enum_iterator::IntoEnumIterator;
use crate::sound::{SoundEngine, Channel};

const SOUNDS_PATH: &str = "./res/sounds";

fn main() {
    let sound_engine = SoundEngine::new(SOUNDS_PATH);
    //let skey = String::from("pulse_open");

    for ch in Channel::into_enum_iter() {
        println!("Playing sine wave on Channel {:?}", ch);
        sound_engine.stop_all();
        sound_engine.play_annoying_sine(ch, 1000);
        sleep(1000);
    }
}

fn sleep(ms: u64) {
    thread::sleep(time::Duration::from_millis(ms));
}