#![allow(dead_code)]

use crate::config::SoundConfig;
use std::path::{Path, PathBuf};
use std::cell::RefCell;
use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;
use std::time::Duration;
use enum_iterator::IntoEnumIterator;
use indexmap::map::IndexMap;
use rodio;
use rodio::source::{Source, Buffered};
use globwalk;
use globset;
use rand;
use rand::Rng;

/// Represents a playback channel for sounds.
#[derive(IntoEnumIterator, Copy, Clone, Debug)]
pub enum Channel {
    Tone,
    Phone1,
    Phone2,
    Phone3,
    Phone4,
    Phone5,
    Phone6,
    Phone7,
    Phone8,
    Soul1,
    Soul2,
    Soul3,
    Soul4,
    Bg1,
    Bg2,
    Bg3,
    Bg4
}

impl Channel {
    pub fn as_index(self) -> usize {
        self as usize
    }
}

impl From<usize> for Channel {    
    fn from(val: usize) -> Self {
        ALL_CHANNELS[val]
    }
}

const ALL_CHANNELS: &[Channel] = { use Channel::*; &[Tone, Phone1, Phone2, Phone3, Phone4, Phone5, Phone6, Phone7, Phone8, Soul1, Soul2, Soul3, Soul4, Bg1, Bg2, Bg3, Bg4] };
const PHONE_CHANNELS: &[Channel] = { use Channel::*; &[Phone1, Phone2, Phone3, Phone4, Phone5, Phone6, Phone7, Phone8] };
const SOUL_CHANNELS: &[Channel] = { use Channel::*; &[Soul1, Soul2, Soul3, Soul4] };
const BG_CHANNELS: &[Channel] = { use Channel::*; &[Bg1, Bg2, Bg3, Bg4] };

const DTMF_COLUMN_FREQUENCIES: &[u32] = &[1209, 1336, 1477, 1633];
const DTMF_ROW_FREQUENCIES: &[u32] = &[697, 770, 852, 941];
const DTMF_DIGITS: &[char] = &['1', '2', '3', 'A', '4', '5', '6', 'B', '7', '8', '9', 'C', '*', '0', '#', 'D'];

/// Converts decibels to amplitude (assuming a normalized signal).
fn db_to_amp(db: f32) -> f32 {
    10.0f32.powf(db / 20.0)
}

pub struct SoundEngine {
    root_path: PathBuf,
    device: rodio::Device,
    channels: RefCell<Vec<SoundChannel>>,
    config: SoundConfig,
    sounds: IndexMap<String, Sound>,
    sound_glob_cache: RefCell<HashMap<String, Vec<usize>>>,
    master_volume: f32
}

struct SoundChannel {
    id: Channel,
    sink: rodio::Sink,
    channel_volume: f32
}

struct Sound {
    path: String,
    src: Buffered<rodio::source::SamplesConverter<rodio::Decoder<BufReader<File>>, i16>>
}

impl Sound {
    fn from_file(path: &Path) -> Self {
        let file = File::open(path).unwrap();
        let src = rodio::Decoder::new(BufReader::new(file)).unwrap().convert_samples::<i16>();

        Self {
            path: String::from(path.to_string_lossy()),
            src: src.buffered()
        }
    }
}

impl SoundEngine {
    pub fn new(root_path: impl Into<String>, config: SoundConfig) -> Self {
        // Load output device
        let device = rodio::default_output_device().expect("No default output device found!");        
        let channels = RefCell::from(Vec::<SoundChannel>::new());
        let master_volume = config.master_volume;

        let mut engine = Self {
            root_path: Path::new(root_path.into().as_str()).canonicalize().unwrap(),
            sounds: IndexMap::new(),
            sound_glob_cache: Default::default(),
            device,
            channels,
            config,
            master_volume
        };

        // Create channels
        for ch in Channel::into_enum_iter() {
            let channel = SoundChannel::new(&engine, ch);
            engine.channels.borrow_mut().push(channel);
        }

        engine.load_sounds();
        engine.set_master_volume(master_volume);

        engine
    }

    fn load_sounds(&mut self) {
        println!("Loading static sound assets...");
        self.sounds.clear();
        self.sound_glob_cache.borrow_mut().clear();
        let search_path = self.root_path.join("**").join("*.{wav,ogg}");
        let search_path_str = search_path.to_str().expect("Failed to create search pattern for sound resources");
        for entry in globwalk::glob(search_path_str).expect("Unable to read search pattern for sound resources") {
            if let Ok(path) = entry {   
                let sound_path = path.path().canonicalize().expect("Unable to expand path");
                let sound_key = sound_path
                .strip_prefix(self.root_path.as_path()).expect("Unable to form sound key from path")
                .with_extension("")
                .to_string_lossy()
                .replace("\\", "/");
                let sound = Sound::from_file(&sound_path);
                self.sounds.insert(sound_key, sound);
            }
        }
    }
}

impl SoundEngine {
    pub fn play(&self, key: &str, channel: Channel, wait: bool, looping: bool, interrupt: bool, speed: f32, volume: f32) {
        let sound = self.find_sound(key);
        match sound {
            Some(sound) => {
                //println!("Playing sound '{}' on channel {:?}", key, channel);
                if interrupt {
                    self.stop(channel);
                }

                let ch = &self.channels.borrow_mut()[channel.as_index()];

                // Queue sound in sink
                ch.queue(sound, looping, speed, volume);
                
                // Optionally wait
                if wait {
                    ch.sink.sleep_until_end();
                }
            },
            None => println!("WARNING: Tried to play nonexistent sound or soundglob '{}'", key)
        }
    }

    pub fn channel_busy(&self, channel: Channel) -> bool {
        let ch = &self.channels.borrow()[channel.as_index()];
        ch.busy()
    }

    pub fn wait(&self, channel: Channel) {
        let ch = &self.channels.borrow()[channel.as_index()];
        ch.sink.sleep_until_end();
    }

    fn find_sound(&self, key: &str) -> Option<&Sound> {
        // Check for exact match
        let sound = self.sounds.get(key);
        match sound {
            // If it exists, just return it right away
            Some(_) => return sound,
            // If not, try a glob match
            None => {
                // First, check if there's a glob match list pre-cached
                let mut glob_cache = self.sound_glob_cache.borrow_mut();
                let glob_list = glob_cache.get(key);
                if let Some(glob_list) = glob_list {
                    let index = rand::thread_rng().gen_range(0, glob_list.len());
                    return Some(&self.sounds.get_index(glob_list[index]).unwrap().1);
                }
                // If not, run the search manually and cache the results
                let glob = globset::GlobBuilder::new(key).literal_separator(true).build();
                if let Ok(glob) = glob {
                    let matcher = glob.compile_matcher();
                    let mut glob_list = Vec::<usize>::new();
                    let sound_iter = self.sounds.iter();
                    for (k, _) in sound_iter {
                        if matcher.is_match(k) {
                            glob_list.push(self.sounds.get_full(k).unwrap().0);
                        }
                    }                    
                    // Cache and pick only if there were results
                    if glob_list.len() > 0 {
                        let index = rand::thread_rng().gen_range(0, glob_list.len());
                        let snd = Some(self.sounds.get_index(glob_list[index]).unwrap().1);
                        glob_cache.insert(key.to_string(), glob_list);
                        return snd;
                    }
                }
                return None;
            }
        }
    }

    pub fn stop_all(&self) {
        for ch in Channel::into_enum_iter() {
            self.stop(ch);            
        }
    }

    pub fn stop(&self, channel: Channel) {
        let mut ch = &mut self.channels.borrow_mut()[channel.as_index()];
        if !ch.sink.empty() {
            ch.sink.stop();
            ch.sink = rodio::Sink::new(&self.device);
        }
    }

    pub fn set_volume(&mut self, channel: Channel, volume: f32) {
        self.channels.borrow_mut()[channel.as_index()].set_volume(volume).update_sink_volume(self.master_volume);
    }

    pub fn volume(&self, channel: Channel) -> f32 {
        self.channels.borrow()[channel.as_index()].volume()
    }

    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }

    pub fn set_master_volume(&mut self, master_volume: f32) {
        self.master_volume = master_volume;
        for ch in Channel::into_enum_iter() {
            self.channels.borrow_mut()[ch.as_index()].update_sink_volume(master_volume);
        }
    }

    pub fn play_dtmf(&self, key: char, duration: f32, volume: f32) -> bool {
        let index = DTMF_DIGITS.iter().position(|&c| c == key);
        let f_row = match index {
            Some(index) => DTMF_ROW_FREQUENCIES[index / 4],
            None => return false
        };
        let f_col = match index {
            Some(index) => DTMF_COLUMN_FREQUENCIES[index % 4],
            None => return false
        };
        self.channels.borrow()[Channel::Tone.as_index()].queue_dtmf(f_row, f_col, duration, volume * self.config.dtmf_volume);
        true
    }

    pub fn play_ringback_tone(&self) {
        self.channels.borrow()[Channel::Tone.as_index()].queue_ringback_tone(db_to_amp(self.config.ringback_tone_gain));
    }

    pub fn play_dial_tone(&self) {
        self.channels.borrow()[Channel::Tone.as_index()].queue_dial_tone(db_to_amp(self.config.dial_tone_gain));
    }

    pub fn play_busy_tone(&self) {
        self.channels.borrow()[Channel::Tone.as_index()].queue_busy_tone(db_to_amp(self.config.busy_tone_gain), false);
    }

    pub fn play_fast_busy_tone(&self) {
        self.channels.borrow()[Channel::Tone.as_index()].queue_busy_tone(db_to_amp(self.config.busy_tone_gain), true);
    }

    pub fn play_off_hook_tone(&self) {
        self.channels.borrow()[Channel::Tone.as_index()].queue_off_hook_tone(db_to_amp(self.config.off_hook_tone_gain));
    }

    pub fn play_panic_tone(&self) {
        self.channels.borrow()[Channel::Tone.as_index()].queue_panic_tone(1.0);
    }
}

impl SoundChannel {
    fn new(engine: &SoundEngine, id: Channel) -> Self {
        let sink = rodio::Sink::new(&engine.device);        
        let ch = Self {
            sink,
            id,
            channel_volume: 1.0
        };
        //ch.update_sink_volume(engine.master_volume);
        ch
    }
}

impl SoundChannel {
    fn update_sink_volume(&mut self, master_volume: f32) -> &mut Self {
        let mixed_vol = master_volume * self.channel_volume;
        self.sink.set_volume(mixed_vol);
        //println!("Sink volume for Channel {:?} is now {}", self.id, self.sink.volume());
        self
    }

    fn volume(&self) -> f32 {
        self.channel_volume
    }

    fn set_volume(&mut self, volume: f32) -> &mut Self {
        self.channel_volume = volume;
        self
    }

    fn busy(&self) -> bool {
        !self.sink.empty()
    }

    fn kill(&self) {
        self.sink.stop();
    }

    fn queue(&self, snd: &Sound, looping: bool, speed: f32, volume: f32) {
        let src = snd.src.clone().amplify(volume);
        let speed_mod = speed != 1.0;
        if looping {
            if speed_mod {
                self.sink.append(src.speed(speed).repeat_infinite());
            } else {
                self.sink.append(src.repeat_infinite());
            }
        } else {
            if speed_mod {
                self.sink.append(src.speed(speed));
            } else {
                self.sink.append(src);
            }
        }
    }

    fn queue_dtmf(&self, f1: u32, f2: u32, duration: f32, volume: f32) {
        let half_volume = volume * 0.5;
        let sine1 = rodio::source::SineWave::new(f1);
        let sine2 = rodio::source::SineWave::new(f2);
        let dtmf_tone = sine1.mix(sine2)
        .take_duration(Duration::from_secs_f32(duration))
        .amplify(half_volume);
        self.sink.append(dtmf_tone);
    }

    fn queue_ringback_tone(&self, volume: f32) {
        const FREQ_RINGBACK_A: u32 = 440;
        const FREQ_RINGBACK_B: u32 = 480;
        let half_volume = volume * 0.5;
        let sine1 = rodio::source::SineWave::new(FREQ_RINGBACK_A);
        let sine2 = rodio::source::SineWave::new(FREQ_RINGBACK_B);
        let ringback_start = 
            sine1.mix(sine2)
            .take_duration(Duration::from_secs(2))
            .amplify(half_volume);
        let ringback_loop = 
            ringback_start.clone()
            .delay(Duration::from_secs(4))
            .repeat_infinite();
        self.sink.append(ringback_start);
        self.sink.append(ringback_loop);
    }

    fn queue_dial_tone(&self, volume: f32) {
        const FREQ_DIAL_A: u32 = 350;
        const FREQ_DIAL_B: u32 = 440;
        let half_volume = volume * 0.5;
        let sine1 = rodio::source::SineWave::new(FREQ_DIAL_A);
        let sine2 = rodio::source::SineWave::new(FREQ_DIAL_B);
        let dial_tone = sine1.mix(sine2).amplify(half_volume).repeat_infinite();
        self.sink.append(dial_tone);
    }

    fn queue_busy_tone(&self, volume: f32, is_fast: bool) {
        const FREQ_BUSY_A: u32 = 480;
        const FREQ_BUSY_B: u32 = 620;
        let half_volume = volume * 0.5;
        let sine1 = rodio::source::SineWave::new(FREQ_BUSY_A);
        let sine2 = rodio::source::SineWave::new(FREQ_BUSY_B);
        let cadence = Duration::from_millis(if is_fast { 250 } else { 500 });
        let busy_start = sine1.mix(sine2).take_duration(cadence).amplify(half_volume);
        let busy_loop = busy_start.clone().delay(cadence).repeat_infinite();
        self.sink.append(busy_start);
        self.sink.append(busy_loop);
    }

    fn queue_off_hook_tone(&self, volume: f32) {
        const FREQ_OFF_HOOK_A: u32 = 1400;
        const FREQ_OFF_HOOK_B: u32 = 2060;
        const FREQ_OFF_HOOK_C: u32 = 2450;
        const FREQ_OFF_HOOK_D: u32 = 2600;
        let quarter_volume = volume * 0.25;
        let sine1 = rodio::source::SineWave::new(FREQ_OFF_HOOK_A);
        let sine2 = rodio::source::SineWave::new(FREQ_OFF_HOOK_B);
        let sine3 = rodio::source::SineWave::new(FREQ_OFF_HOOK_C);
        let sine4 = rodio::source::SineWave::new(FREQ_OFF_HOOK_D);
        let cadence = Duration::from_millis(100);
        let off_hook_start = 
            sine1.mix(sine2).mix(sine3).mix(sine4)
            .take_duration(cadence)
            .amplify(quarter_volume)
            .buffered();
        let off_hook_loop = off_hook_start.clone().delay(cadence).repeat_infinite();
        self.sink.append(off_hook_start);
        self.sink.append(off_hook_loop);
    }

    fn queue_panic_tone(&self, volume: f32) {
        const FREQ_PANIC_A: u32 = 720;
        const FREQ_PANIC_B: u32 = 900;
        let half_volume = volume * 0.5;
        let sine1 = rodio::source::SineWave::new(FREQ_PANIC_A);
        let sine2 = rodio::source::SineWave::new(FREQ_PANIC_B);
        let cadence = Duration::from_millis(375);
        let busy_start = sine1.mix(sine2).take_duration(cadence).amplify(half_volume);
        let busy_loop = busy_start.clone().delay(cadence).repeat_infinite();
        self.sink.append(busy_start);
        self.sink.append(busy_loop);
    }
}