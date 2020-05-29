#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::cell::RefCell;
use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;
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

pub struct SoundEngine {
    root_path: PathBuf,
    device: rodio::Device,
    channels: RefCell<Vec<SoundChannel>>,
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
    pub fn new(root_path: impl Into<String>) -> Self {
        // Load output device
        let device = rodio::default_output_device().expect("No default output device found!");        
        let channels = RefCell::from(Vec::<SoundChannel>::new());

        let mut engine = Self {
            root_path: Path::new(root_path.into().as_str()).canonicalize().unwrap(),
            sounds: IndexMap::new(),
            sound_glob_cache: Default::default(),
            device,
            channels,
            master_volume: 1.0
        };

        // Create channels
        for ch in Channel::into_enum_iter() {
            let channel = SoundChannel::new(&engine, ch);
            engine.channels.borrow_mut().push(channel);
        }

        engine.load_sounds();

        engine
    }

    fn load_sounds(&mut self) {
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
    pub fn play(&self, key: &str, channel: Channel, wait: bool, looping: bool, interrupt: bool, speed: f32) {
        let sound = self.find_sound(key);
        match sound {
            Some(sound) => {
                //println!("Playing sound '{}' on channel {:?}", key, channel);
                if interrupt {
                    self.stop(channel);
                }

                let ch = &self.channels.borrow_mut()[channel.as_index()];

                // Queue sound in sink
                ch.queue(sound, looping, speed);
                
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
    }

    pub fn play_annoying_sine(&self, channel: Channel, f: u32) {
        let sink = &self.channels.borrow()[channel.as_index()].sink;
        let src = rodio::source::SineWave::new(f);
        sink.append(src);
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

    fn queue(&self, snd: &Sound, looping: bool, speed: f32) {
        let src = snd.src.clone();
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
}