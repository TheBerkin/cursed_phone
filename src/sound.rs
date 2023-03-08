#![allow(dead_code)]

use crate::config::*;
use std::path::{Path, PathBuf};
use std::cell::RefCell;
use std::rc::Rc;
use std::fs::File;
use std::io::BufReader;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use enum_iterator::Sequence;
use indexmap::map::IndexMap;
use mlua::FromLua;
use rodio;
use rodio::buffer::SamplesBuffer;
use rodio::source::{Source, Buffered};
use globwalk;
use globset;
use rand;
use rand::Rng;
use log::{info, warn};

/// Represents a playback channel for sounds.
#[derive(Sequence, Copy, Clone, Debug, PartialEq)]
pub enum Channel {
    /// Channel for incoming telephony signal tones.
    SignalIn,
    /// Channel for incoming comfort noise.
    NoiseIn,
    /// Channel for outgoing telephony signal tones.
    SignalOut,
    /// Phone Channel 1.
    Phone01,
    /// Phone Channel 2.
    Phone02,
    /// Phone Channel 3.
    Phone03,
    /// Phone Channel 4.
    Phone04,
    /// Phone Channel 5.
    Phone05,
    /// Phone Channel 6.
    Phone06,
    /// Phone Channel 7.
    Phone07,
    /// Phone Channel 8.
    Phone08,
    /// Phone Channel 9.
    Phone09,
    /// Phone Channel 10.
    Phone10,
    /// Soul Channel 1.
    Soul1,
    /// Soul Channel 2.
    Soul2,
    /// Soul Channel 3.
    Soul3,
    /// Soul Channel 4.
    Soul4,
    /// Background Channel 1.
    Bg1,
    /// Background Channel 2.
    Bg2,
    /// Background Channel 3.
    Bg3,
    /// Background Channel 4.
    Bg4,
    /// Debug channel.
    Debug,
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

const ALL_CHANNELS: &[Channel] = { use Channel::*; &[SignalIn, NoiseIn, SignalOut, Phone01, Phone02, Phone03, Phone04, Phone05, Phone06, Phone07, Phone08, Phone09, Phone10, Soul1, Soul2, Soul3, Soul4, Bg1, Bg2, Bg3, Bg4, Debug] };
const PHONE_CHANNELS: &[Channel] = { use Channel::*; &[Phone01, Phone02, Phone03, Phone04, Phone05, Phone06, Phone07, Phone08, Phone09, Phone10] };
const SOUL_CHANNELS: &[Channel] = { use Channel::*; &[Soul1, Soul2, Soul3, Soul4] };
const BG_CHANNELS: &[Channel] = { use Channel::*; &[Bg1, Bg2, Bg3, Bg4] };

// DTMF tone constants
const DTMF_COLUMN_FREQUENCIES: &[f32] = &[1209.0, 1336.0, 1477.0, 1633.0];
const DTMF_ROW_FREQUENCIES: &[f32] = &[697.0, 770.0, 852.0, 941.0];
const DTMF_DIGITS: &[char] = &['1', '2', '3', 'A', '4', '5', '6', 'B', '7', '8', '9', 'C', '*', '0', '#', 'D'];

// Special Information Tone constants
const SIT_FREQS_FIRST: (u32, u32) = (914, 985);
const SIT_FREQS_SECOND: (u32, u32) = (1371, 1429);
const SIT_FREQS_THIRD: (u32, u32) = (1777, 1777);
const SIT_SHORT_SEG_MS: u64 = 276;
const SIT_LONG_SEG_MS: u64 = 380;

/// Special Information Tone (SIT) types.
///
/// Descriptions from [Wikipedia](https://en.wikipedia.org/wiki/Special_information_tone)
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum SpecialInfoTone {
    /// Unassigned N11 ode, CLASS code, or prefix.
    VacantCode = 0,
    /// Incomplete digits, internal office or feature failure (local office).
    ReorderIntra = 1,
    /// Call failure, no wink or partial digits received (distant office).
    ReorderInter = 2,
    /// All circuits busy (local office).
    NoCircuitIntra = 3,
    /// All circuits busy (distant office).
    NoCircuitInter = 4,
    /// Number changed or disconnected.
    Intercept = 5,
    /// General misdialing, coin deposit required or other failure.
    Ineffective = 6,
    /// Reserved for future use.
    Reserved = 7
}

impl SpecialInfoTone {
    fn as_segments(self) -> (SitSegment, SitSegment, SitSegment) {
        use SitSegment::*;
        use SitSegmentLength::*;
        match self {
            SpecialInfoTone::VacantCode => (High(Long), Low(Short), Low(Long)),
            SpecialInfoTone::ReorderIntra => (Low(Short), High(Long), Low(Long)),
            SpecialInfoTone::ReorderInter => (High(Short), Low(Long), Low(Long)),
            SpecialInfoTone::NoCircuitIntra => (High(Long), High(Long), Low(Long)),
            SpecialInfoTone::NoCircuitInter => (Low(Long), Low(Long), Low(Long)),
            SpecialInfoTone::Intercept => (Low(Short), Low(Short), Low(Long)),
            SpecialInfoTone::Ineffective => (Low(Long), High(Short), Low(Long)),
            SpecialInfoTone::Reserved => (High(Short), High(Short), Low(Long)),
        }
    }
}

impl From<u8> for SpecialInfoTone {
    fn from(val: u8) -> Self {
        match val {
            0 => SpecialInfoTone::VacantCode,
            1 => SpecialInfoTone::ReorderIntra,
            2 => SpecialInfoTone::ReorderInter,
            3 => SpecialInfoTone::NoCircuitIntra,
            4 => SpecialInfoTone::NoCircuitInter,
            5 => SpecialInfoTone::Intercept,
            6 => SpecialInfoTone::Ineffective,
            7 | _ => SpecialInfoTone::Reserved
        }
    }
}

enum SitSegmentLength {
    Short,
    Long
}

impl SitSegmentLength {
    fn as_ms(self) -> u64 {
        use SitSegmentLength::*;
        match self {
            Short => SIT_SHORT_SEG_MS,
            Long => SIT_LONG_SEG_MS
        }
    }
}

enum SitSegment {
    High(SitSegmentLength),
    Low(SitSegmentLength)
}

/// Converts decibels to amplitude (assuming a normalized signal).
fn db_to_amp(db: f32) -> f32 {
    10.0f32.powf(db / 20.0)
}

pub struct SoundEngine {
    sounds_root_path: PathBuf,
    sound_banks_root_path: PathBuf,
    stream: rodio::OutputStream,
    stream_handle: rodio::OutputStreamHandle,
    channels: RefCell<Vec<SoundChannel>>,
    config: Rc<CursedConfig>,
    static_sounds: SoundBank,
    sound_banks: IndexMap<String, Rc<RefCell<SoundBank>>>,
    master_volume: f32
}

struct SoundChannel {
    id: Channel,
    sink: rodio::Sink,
    volume_master: f32,
    volume_channel: f32,
    volume_fade: f32,
    muted: bool,
}

struct Sound {
    path: String,
    src: Buffered<SamplesBuffer<i16>>,
}

impl Sound {
    fn from_file(path: &Path) -> Self {
        let file = File::open(path).unwrap();
        let decoder = rodio::Decoder::new(BufReader::new(file)).unwrap().convert_samples::<i16>();
        let sample_rate = decoder.sample_rate();
        let channels = decoder.channels();
        let samples = decoder.collect::<Vec<i16>>();
        let src = SamplesBuffer::new(channels, sample_rate, samples).buffered();
        
        Self {
            path: String::from(path.to_string_lossy()),
            src: src
        }
    }

    fn duration(&self) -> Option<Duration> {
        self.src.total_duration()
    }
}

#[derive(Clone, Copy)]
pub struct SoundPlayOptions {
    pub volume: f32,
    pub speed: f32,
    pub looping: bool,
    pub skip: SoundPlaySkip,
    pub take: Option<Duration>,
    pub delay: Option<Duration>,
    pub fadein: Duration,
}

impl Default for SoundPlayOptions {
    fn default() -> Self {
        Self { 
            volume: 1.0, 
            speed: 1.0, 
            looping: false, 
            skip: Default::default(), 
            take: Default::default(),
            delay: Default::default(),
            fadein: Default::default(),
        }
    }
}

#[derive(Clone, Copy)]
pub enum SoundPlaySkip {
    By(Duration),
    Random
}

impl Default for SoundPlaySkip {
    fn default() -> Self {
        Self::By(Duration::ZERO)
    }
}

impl<'lua> FromLua<'lua> for SoundPlaySkip {
    fn from_lua(lua_value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        Ok(match lua_value {
            mlua::Value::Nil => Default::default(),
            mlua::Value::Integer(secs) => Self::By(Duration::from_secs(secs as u64)),
            mlua::Value::Number(secs) => Self::By(Duration::from_secs_f64(secs)),
            mlua::Value::String(kw) => match kw.to_str() {
                Ok("random") => Self::Random,
                Ok(kw_other) => return Err(mlua::Error::FromLuaConversionError { from: "string", to: stringify!(SoundPlaySkip), message: Some(format!("invalid sound skip keyword: \"{}\"", kw_other)) }),
                Err(_) => return Err(mlua::Error::FromLuaConversionError { from: "string", to: stringify!(SoundPlaySkip), message: None })
            },
            other => return Err(mlua::Error::FromLuaConversionError { from: other.type_name(), to: stringify!(SoundPlaySkip), message: None })
        })
    }
}

#[derive(Hash, Eq, PartialEq, Debug)]
pub struct SoundBankUser(pub usize);

struct SoundBank {
    name: String,
    root_dir: PathBuf,
    sounds: IndexMap<String, Rc<Sound>>,
    sound_glob_cache: RefCell<HashMap<String, Vec<usize>>>,
    users: HashSet<SoundBankUser>,
}

pub struct PlayedSoundInfo {
    pub duration: Option<Duration>
}

impl SoundBank {
    pub fn from_dir(name: String, root_dir: &Path) -> Self {   
        
        let mut bank = Self {
            name,
            root_dir: PathBuf::from(root_dir),
            sounds: Default::default(),
            sound_glob_cache: Default::default(),
            users: Default::default()
        };

        match root_dir.canonicalize() {
            Ok(root_dir) => {
                bank.sounds.clear();
                bank.sound_glob_cache.borrow_mut().clear();
                let search_path = root_dir.join("**").join("*.{wav,ogg}");
                let search_path_str = search_path.to_str().expect("Failed to create search pattern for sound bank");
                for entry in globwalk::glob(search_path_str).expect("Unable to read search pattern for sound bank") {
                    if let Ok(path) = entry {
                        let sound_path = path.path().canonicalize().expect("Unable to expand path");
                        let sound_key = sound_path
                        .strip_prefix(&root_dir).expect("Unable to form sound key from path")
                        .with_extension("")
                        .to_string_lossy()
                        .replace("\\", "/");
                        let sound = Sound::from_file(&sound_path);
                        bank.sounds.insert(sound_key, Rc::new(sound));
                    }
                }
            }
            Err(err) => warn!("Failed to load soundbank content at '{:?}': {}", root_dir.as_os_str(), err),
        }

        bank
    }

    pub fn add_user(&mut self, user: SoundBankUser) -> bool {
        self.users.insert(user)
    }

    pub fn remove_user(&mut self, user: &SoundBankUser) -> bool {
        self.users.remove(user)
    }

    pub fn has_user(&self, user: &SoundBankUser) -> bool {
        self.users.contains(user)
    }

    pub fn user_count(&self) -> usize {
        self.users.len()
    }

    pub fn find_sound(&self, key: &str) -> Option<Rc<Sound>> {
        // Check for exact match
        let sound = self.sounds.get(key);
        match sound {
            // If it exists, just return it right away
            Some(sound) => return Some(Rc::clone(sound)),
            // If not, try a glob match
            None => {
                // First, check if there's a glob match list pre-cached
                let mut glob_cache = self.sound_glob_cache.borrow_mut();
                let glob_list = glob_cache.get(key);
                if let Some(glob_list) = glob_list {
                    let index = rand::thread_rng().gen_range(0..glob_list.len());
                    return Some(Rc::clone(&self.sounds.get_index(glob_list[index]).unwrap().1));
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
                        let index = rand::thread_rng().gen_range(0..glob_list.len());
                        let sound = Some(self.sounds.get_index(glob_list[index]).unwrap().1);
                        glob_cache.insert(key.to_string(), glob_list);
                        return Some(Rc::clone(sound.unwrap()));
                    }
                }
                return None;
            }
        }
    }
}

impl SoundEngine {
    pub fn new(sounds_root_path: &str, sound_banks_root_path: &str, config: &Rc<CursedConfig>) -> Self {
        // Load output device
        let (stream, stream_handle) = rodio::OutputStream::try_default().expect("Failed to open audio output device!");
        let channels = RefCell::from(Vec::<SoundChannel>::new());
        let config = Rc::clone(config);
        let master_volume = config.sound.master_volume;
        let sounds_root_path = Path::new(sounds_root_path);

        info!("Loading static sound resources...");
        let static_sounds = SoundBank::from_dir("[static]".to_owned(),sounds_root_path);

        let mut engine = Self {
            sounds_root_path: sounds_root_path.canonicalize().expect("Unable to expand static sound root path"),
            sound_banks_root_path: Path::new(sound_banks_root_path).canonicalize().expect("Unable to expand soundbank root path"),
            sound_banks: Default::default(),
            static_sounds,
            stream,
            stream_handle,
            channels,
            config,
            master_volume
        };

        // Create channels
        for ch in enum_iterator::all::<Channel>() {
            let channel = SoundChannel::new(&engine, ch);
            engine.channels.borrow_mut().push(channel);
        }

        engine.set_master_volume(master_volume);

        engine
    }
}

impl SoundEngine {
    pub fn play(&self, key: &str, channel: Channel, wait: bool, interrupt: bool, opts: SoundPlayOptions) -> Option<PlayedSoundInfo> {
        let sound = self.find_sound(key);
        match sound {
            Some(sound) => {
                if interrupt {
                    self.stop(channel);
                }

                let ch = &mut self.channels.borrow_mut()[channel.as_index()];
                let info = PlayedSoundInfo {
                    duration: sound.duration()
                };

                // Queue sound in sink
                ch.set_volume(VolumeLayer::Fade, 1.0);
                ch.queue(sound, opts);
                
                // Optionally wait
                if wait {
                    ch.sink.sleep_until_end();
                }

                Some(info)
            },
            None => {
                warn!("WARNING: Tried to play nonexistent sound or soundglob '{}'", key);
                None
            }
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

    fn get_sound_bank(&self, name: &str) -> Option<Rc<RefCell<SoundBank>>> {
        if let Some(bank) = self.sound_banks.get(name) {
            return Some(Rc::clone(bank));
        }
        None
    }

    pub fn add_sound_bank_user(&mut self, name: &str, user: SoundBankUser) -> bool {
        if let Some(bank) = self.get_sound_bank(name) {
            return bank.borrow_mut().add_user(user);
        }

        info!("Loading sound bank: '{}'", name);
        let bank_path = self.sound_banks_root_path.join(name);
        let bank_path = bank_path.as_path();
        let mut bank = SoundBank::from_dir(name.to_owned(), bank_path);
        bank.add_user(user);

        self.sound_banks.insert(name.to_owned(), Rc::new(RefCell::new(bank)));
        true
    }

    pub fn sound_bank_used_by(&self, name: &str, user: &SoundBankUser) -> bool {
        if let Some(bank) = self.get_sound_bank(name) {
            return bank.borrow().has_user(&user);
        }
        false
    }

    pub fn remove_sound_bank_user(&mut self, name: &str, user: SoundBankUser, unload_if_userless: bool) -> bool {
        if let Some(bank) = self.get_sound_bank(name) {
            let removed = bank.borrow_mut().remove_user(&user);
            if unload_if_userless && bank.borrow().user_count() == 0 {
                info!("Unloading sound bank: '{}'", name);
                self.sound_banks.remove(name);
            }
            return removed;
        }
        false
    }

    fn find_sound(&self, key: &str) -> Option<Rc<Sound>> {
        // See if it's a soundbank sound
        if key.starts_with("$") {
            if let Some(separator_index) = key.find('/') {
                let soundbank_name = &key[1..separator_index];
                if let Some(bank) = self.get_sound_bank(soundbank_name) {
                    let key = &key[separator_index + 1 ..];
                    return bank.borrow().find_sound(key)
                }
            }
        }
        // Find as static sound
        self.static_sounds.find_sound(key)
    }

    pub fn stop_all(&self) {
        for ch in enum_iterator::all::<Channel>() {
            self.stop(ch);            
        }
    }

    pub fn stop_all_except(&self, except: Channel) {
        for ch in enum_iterator::all::<Channel>() {
            if ch == except {
                continue;
            }
            self.stop(ch);            
        }
    }

    pub fn stop_all_nonsignal(&self) {
        for ch in enum_iterator::all::<Channel>() {
            match ch {
                Channel::NoiseIn | Channel::SignalIn => continue,
                _ => self.stop(ch)
            }    
        }
    }

    pub fn stop(&self, channel: Channel) {
        let mut ch = &mut self.channels.borrow_mut()[channel.as_index()];
        if !ch.sink.empty() {
            ch.sink.stop();
            ch.sink = rodio::Sink::try_new(&self.stream_handle).expect("Failed to rebuild sound channel");
        }
    }

    pub fn is_muted(&self, channel: Channel) -> bool {
        self.channels.borrow()[channel.as_index()].muted()
    }

    pub fn set_muted(&mut self, channel: Channel, muted: bool) {
        self.channels.borrow_mut()[channel.as_index()].set_muted(muted);
    }

    pub fn set_volume(&mut self, channel: Channel, volume: f32) {
        self.channels.borrow_mut()[channel.as_index()].set_volume(VolumeLayer::Channel, volume);
    }

    pub fn volume(&self, channel: Channel) -> f32 {
        self.channels.borrow()[channel.as_index()].volume(VolumeLayer::Channel)
    }

    pub fn set_fade_volume(&mut self, channel: Channel, volume: f32) {
        self.channels.borrow_mut()[channel.as_index()].set_volume(VolumeLayer::Fade, volume);
    }

    pub fn fade_volume(&self, channel: Channel) -> f32 {
        self.channels.borrow()[channel.as_index()].volume(VolumeLayer::Fade)
    }

    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }

    pub fn set_master_volume(&mut self, master_volume: f32) {
        self.master_volume = master_volume;
        for ch in enum_iterator::all::<Channel>() {
            self.channels.borrow_mut()[ch.as_index()].set_volume(VolumeLayer::Master, master_volume);
        }
    }

    pub fn play_dtmf(&self, key: char, dur: Duration, volume: f32) -> bool {
        let index = DTMF_DIGITS.iter().position(|&c| c == key);
        let f_row = match index {
            Some(index) => DTMF_ROW_FREQUENCIES[index / 4],
            None => return false
        };
        let f_col = match index {
            Some(index) => DTMF_COLUMN_FREQUENCIES[index % 4],
            None => return false
        };
        self.channels.borrow()[Channel::SignalOut.as_index()].queue_dtmf(f_row, f_col, dur, volume * self.config.sound.dtmf_volume);
        true
    }

    // TODO: Cache dB-to-amplitude conversions for call progress tones

    pub fn play_ringback_tone(&self) {
        self.stop(Channel::SignalIn);
        self.channels.borrow()[Channel::SignalIn.as_index()].queue_ringback_tone(db_to_amp(self.config.sound.ringback_tone_gain));
    }

    pub fn play_dial_tone(&self) {
        self.stop(Channel::SignalIn);
        self.channels.borrow()[Channel::SignalIn.as_index()].queue_dial_tone(db_to_amp(self.config.sound.dial_tone_gain));
    }

    pub fn play_busy_tone(&self) {
        self.stop(Channel::SignalIn);
        self.channels.borrow()[Channel::SignalIn.as_index()].queue_busy_tone(db_to_amp(self.config.sound.busy_tone_gain), false);
    }

    pub fn play_fast_busy_tone(&self) {
        self.stop(Channel::SignalIn);
        self.channels.borrow()[Channel::SignalIn.as_index()].queue_busy_tone(db_to_amp(self.config.sound.busy_tone_gain), true);
    }

    pub fn play_off_hook_tone(&self) {
        self.stop(Channel::SignalIn);
        self.channels.borrow()[Channel::SignalIn.as_index()].queue_off_hook_tone(db_to_amp(self.config.sound.off_hook_tone_gain));
    }

    pub fn play_panic_tone(&self) {
        self.stop(Channel::SignalIn);
        self.channels.borrow()[Channel::Debug.as_index()].queue_panic_tone(1.0);
    }

    pub fn play_special_info_tone(&self, sit: SpecialInfoTone) {
        let (first, second, third) = sit.as_segments();
        self.stop(Channel::SignalIn);
        self.channels.borrow()[Channel::SignalIn.as_index()].queue_special_info_tone(
            first, 
            second, 
            third, 
            db_to_amp(self.config.sound.special_info_tone_gain));
    }
}

#[derive(Clone, Copy, PartialEq)]
enum VolumeLayer {
    /// Master volume mix
    Master,
    /// Channel volume mix
    Channel,
    /// Fade volume mix
    Fade,
}

impl SoundChannel {
    fn new(engine: &SoundEngine, id: Channel) -> Self {
        let sink = rodio::Sink::try_new(&engine.stream_handle).expect("Failed to create sound channel");        
        let ch = Self {
            sink,
            id,
            volume_master: 1.0,
            volume_channel: 1.0,
            volume_fade: 1.0,
            muted: false,
        };
        //ch.update_sink_volume(engine.master_volume);
        ch
    }
}

impl SoundChannel {
    fn update_sink_volume(&mut self) -> &mut Self {
        self.sink.set_volume(self.mixed_volume());
        // trace!("Sink volume for Channel {:?} is now {}", self.id, self.sink.volume());
        self
    }

    fn mixed_volume(&self) -> f32 {
        self.volume_master * self.volume_channel * self.volume_fade * if self.muted { 0.0 } else { 1.0 }
    }

    fn volume(&self, layer: VolumeLayer) -> f32 {
        match layer {
            VolumeLayer::Master => self.volume_master,
            VolumeLayer::Channel => self.volume_channel,
            VolumeLayer::Fade => self.volume_fade,
        }
    }

    fn set_volume(&mut self, layer: VolumeLayer, volume: f32) -> &mut Self {
        let dst_volume = match layer {
            VolumeLayer::Master => &mut self.volume_master,
            VolumeLayer::Channel => &mut self.volume_channel,
            VolumeLayer::Fade => &mut self.volume_fade,
        };
        *dst_volume = volume;
        self.update_sink_volume();
        self
    }

    fn muted(&self) -> bool {
        self.muted
    }

    fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
        self.update_sink_volume();
    }

    fn busy(&self) -> bool {
        !self.sink.empty()
    }

    fn kill(&self) {
        self.sink.stop();
    }

    fn queue(&self, snd: Rc<Sound>, opts: SoundPlayOptions) {
        let src = snd.src.clone().amplify(opts.volume);
        if let Some(delay) = opts.delay {
            self.sink.append(rodio::source::Empty::<i16>::new().delay(delay))
        }
        let skip = match &opts.skip {
            SoundPlaySkip::By(duration) => *duration,
            SoundPlaySkip::Random => Duration::from_secs_f64(rand::thread_rng().gen_range(0.0 ..= snd.duration().unwrap_or_default().as_secs_f64())),
        };
        let is_nonstandard_speed = opts.speed != 1.0;
        if let Some(take) = opts.take {
            if opts.looping {
                if is_nonstandard_speed {
                    self.sink.append(src.repeat_infinite().skip_duration(skip).take_duration(take).speed(opts.speed).fade_in(opts.fadein));
                } else {
                    self.sink.append(src.repeat_infinite().skip_duration(skip).take_duration(take).fade_in(opts.fadein));
                }
            } else {
                if is_nonstandard_speed {
                    self.sink.append(src.skip_duration(skip).take_duration(take).speed(opts.speed).fade_in(opts.fadein));
                } else {
                    self.sink.append(src.skip_duration(skip).take_duration(take).fade_in(opts.fadein));
                }
            }
        } else {
            if opts.looping {
                if is_nonstandard_speed {
                    self.sink.append(src.repeat_infinite().skip_duration(skip).speed(opts.speed).fade_in(opts.fadein));
                } else {
                    self.sink.append(src.repeat_infinite().skip_duration(skip).fade_in(opts.fadein));
                }
            } else {
                if is_nonstandard_speed {
                    self.sink.append(src.skip_duration(skip).speed(opts.speed).fade_in(opts.fadein));
                } else {
                    self.sink.append(src.skip_duration(skip).fade_in(opts.fadein));
                }
            }
        }
    }

    fn queue_dtmf(&self, f1: f32, f2: f32, dur: Duration, volume: f32) {
        let half_volume = volume * 0.5;
        let sine1 = rodio::source::SineWave::new(f1);
        let sine2 = rodio::source::SineWave::new(f2);
        let dtmf_tone = sine1.mix(sine2)
        .take_duration(dur)
        .amplify(half_volume);
        self.sink.append(dtmf_tone);
    }

    fn queue_ringback_tone(&self, volume: f32) {
        const FREQ_RINGBACK_A: f32 = 440.0;
        const FREQ_RINGBACK_B: f32 = 480.0;
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
        const FREQ_DIAL_A: f32 = 350.0;
        const FREQ_DIAL_B: f32 = 440.0;
        let half_volume = volume * 0.5;
        let sine1 = rodio::source::SineWave::new(FREQ_DIAL_A);
        let sine2 = rodio::source::SineWave::new(FREQ_DIAL_B);
        let dial_tone = sine1.mix(sine2).amplify(half_volume).repeat_infinite();
        self.sink.append(dial_tone);
    }

    fn queue_busy_tone(&self, volume: f32, is_fast: bool) {
        const FREQ_BUSY_A: f32 = 480.0;
        const FREQ_BUSY_B: f32 = 620.0;
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
        const FREQ_OFF_HOOK_A: f32 = 1400.0;
        const FREQ_OFF_HOOK_B: f32 = 2060.0;
        const FREQ_OFF_HOOK_C: f32 = 2450.0;
        const FREQ_OFF_HOOK_D: f32 = 2600.0;
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
        const FREQ_PANIC_A: f32 = 720.0;
        const FREQ_PANIC_B: f32 = 900.0;
        let half_volume = volume * 0.5;
        let sine1 = rodio::source::SineWave::new(FREQ_PANIC_A);
        let sine2 = rodio::source::SineWave::new(FREQ_PANIC_B);
        let cadence = Duration::from_millis(375);
        let busy_start = sine1.mix(sine2).take_duration(cadence).amplify(half_volume);
        let busy_loop = busy_start.clone().delay(cadence).repeat_infinite();
        self.sink.append(busy_start);
        self.sink.append(busy_loop);
    }

    fn queue_special_info_tone(&self, first: SitSegment, second: SitSegment, third: SitSegment, volume: f32) {
        use SitSegment::*;
        let (f1, d1) = match first {
            Low(len) => (SIT_FREQS_FIRST.0, len.as_ms()),
            High(len) => (SIT_FREQS_FIRST.1, len.as_ms())
        };
        let (f2, d2) = match second {
            Low(len) => (SIT_FREQS_SECOND.0, len.as_ms()),
            High(len) => (SIT_FREQS_SECOND.1, len.as_ms())
        };
        let (f3, d3) = match third {
            Low(len) => (SIT_FREQS_THIRD.0, len.as_ms()),
            High(len) => (SIT_FREQS_THIRD.1, len.as_ms())
        };
        let sine1 = rodio::source::SineWave::new(f1 as f32)
            .take_duration(Duration::from_millis(d1))
            .amplify(volume);
        let sine2 = rodio::source::SineWave::new(f2 as f32)
            .take_duration(Duration::from_millis(d2))
            .amplify(volume);
        let sine3 = rodio::source::SineWave::new(f3 as f32)
            .take_duration(Duration::from_millis(d3))
            .amplify(volume);
        self.sink.append(sine1);
        self.sink.append(sine2);
        self.sink.append(sine3);
    }
}