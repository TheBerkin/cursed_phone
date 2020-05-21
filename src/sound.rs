#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::cell::RefCell;
use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;
use enum_iterator::IntoEnumIterator;
use rodio;
use rodio::source::{Source, Buffered};
use globwalk;

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
    sounds: HashMap<String, Sound>,
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
    pub fn new(root_path: impl Into<String>) -> SoundEngine {
        // Load output device
        let device = rodio::default_output_device().expect("No default output device found!");

        let channels = RefCell::from(Vec::<SoundChannel>::new());

        let mut engine = SoundEngine {
            root_path: Path::new(root_path.into().as_str()).canonicalize().unwrap(),
            sounds: HashMap::new(),
            device,
            channels,
            master_volume: 1.0
        };

        // Create channels
        for _ in Channel::into_enum_iter() {
            engine.create_channel();
        }

        engine.load_sounds();

        engine
    }

    fn load_sounds(&mut self) {
        let search_path = self.root_path.join("**").join("*.{wav,ogg}");
        let search_path_str = search_path.to_str().expect("Failed to create search pattern for sound resources");
        println!("{}", search_path_str);
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
    fn create_channel(&self) {
        let channel = SoundChannel::new(self, Channel::from(self.channels.borrow().len()));
        self.channels.borrow_mut().push(channel);
    }

    pub fn play(&self, key: &String, channel: Channel) {
        let sound = self.sounds.get(key);
        match sound {
            Some(sound) => {
                println!("Playing sound '{}' on channel {:?}", key, channel);
                let channel = &self.channels.borrow_mut()[channel.as_index()];
                channel.play(sound)
            },
            None => println!("WARNING: Tried to play nonexistent sound '{}'", key)
        }
    }

    pub fn stop_all(&self) {
        for ch in Channel::into_enum_iter() {
            self.stop(ch);
        }
    }

    pub fn stop(&self, channel: Channel) {
        self.channels.borrow()[channel.as_index()].stop();
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
        let ch = &self.channels.borrow()[channel.as_index()];
        ch.sink.stop();
        let src = rodio::source::SineWave::new(f);
        ch.sink.append(src);
    }
}

impl SoundChannel {
    fn new(engine: &SoundEngine, id: Channel) -> Self {
        let mut ch = Self {
            sink: rodio::Sink::new(&engine.device),
            id,
            channel_volume: 1.0
        };
        ch.update_sink_volume(engine.master_volume);
        ch
    }
}

impl SoundChannel {
    fn update_sink_volume(&mut self, master_volume: f32) -> &mut Self {
        let mixed_vol = master_volume * self.channel_volume;
        self.sink.set_volume(mixed_vol);
        println!("Sink volume for Channel {:?} is now {}", self.id, self.sink.volume());
        self
    }

    fn volume(&self) -> f32 {
        self.channel_volume
    }

    fn set_volume(&mut self, volume: f32) -> &mut Self {
        self.channel_volume = volume;
        self
    }

    fn stop(&self) {
        self.sink.stop();
    }

    fn play(&self, snd: &Sound) {
        self.sink.stop();
        self.sink.append(snd.src.clone());
        self.sink.play();
    }

    fn play_next(&self, snd: &Sound) {
        self.sink.append(snd.src.clone());
        self.sink.play();
    }
}