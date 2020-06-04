use std::fs;
use serde::Deserialize;
use toml;

#[allow(non_camel_case_types)]
type ms = u64;

#[serde(rename_all = "kebab-case")]
#[derive(Deserialize, Clone, Debug)]
pub struct CursedConfig {
    /// Phone type. Mainly affects which inputs control dialing.
    /// See table for supported values.
    /// 
    /// |Type Name    |Description              |
    /// |:------------|:------------------------|
    /// |`"rotary"`   |Rotary phone (pulse dial)|
    /// |`"touchtone"`|Touch-tone (keypad dial) |
    /// |`"unknown"`  |Unknown/other phone type |
    pub phone_type: String,

    /// **Post Dial Delay (PDD)**
    /// 
    /// Delay (in seconds) to wait after the last digit is dialed, 
    /// before the phone attempts to place the call.
    pub pdd: f32,

    /// Delay (in seconds) before off-hook intercept message is played.
    pub off_hook_delay: f32,

    /// Maximum seconds between pulses for switch-hook dialing.
    pub manual_pulse_interval: f32,

    /// Number of seconds phone must be on the hook to end the call.
    /// 
    /// **Must** be greater than manual_pulse_interval!
    /// 
    /// (Only used if `enable_switch_hook_dialing == true`)
    pub hangup_delay: f32,

    /// Enables ringer.
    pub enable_ringer: Option<bool>,

    /// Enables vibration.
    pub enable_vibration: Option<bool>,

    /// Enables motion sensor.
    pub enable_motion_sensor: Option<bool>,

    /// Enables switch-hook dialing.
    pub enable_switch_hook_dialing: Option<bool>,

    /// Sound configuration.
    pub sound: SoundConfig,

    /// GPIO configuration.
    pub gpio: GpioConfig,

    /// Debug feature configuration.
    pub debug: Option<DebugConfig>
}

#[serde(rename_all = "kebab-case")]
#[derive(Deserialize, Copy, Clone, Debug)]
pub struct SoundConfig {
    /// Initial master volume.
    pub master_volume: f32,
    pub dial_tone_gain: f32,
    pub ringback_tone_gain: f32,
    pub busy_tone_gain: f32,
    pub off_hook_tone_gain: f32
}

#[serde(rename_all = "kebab-case")]
#[derive(Deserialize, Copy, Clone, Debug)]
pub struct GpioConfig {
    /// Input configuration.
    pub inputs: GpioInputsConfig,
    /// Output configuration.
    pub outputs: GpioOutputsConfig
}

#[serde(rename_all = "kebab-case")]
#[derive(Deserialize, Copy, Clone, Debug)]
pub struct GpioInputsConfig {
    /// BCM pin number of switch hook input.
    pub pin_hook: u8,
    /// Bounce time (ms) of switch hook input.
    pub pin_hook_bounce_ms: Option<ms>,
    /// BCM pin number of dial pulse input.
    pub pin_dial_pulse: Option<u8>,
    /// Bounce time (ms) of dial pulse input.
    pub pin_dial_pulse_bounce_ms: Option<ms>,
    /// BCM pin number of dial switch input.
    pub pin_dial_switch: Option<u8>,
    /// Bounce time (ms) of dial switch input.
    pub pin_dial_switch_bounce_ms: Option<ms>,
    /// BCM pin number of motion sensor input.
    pub pin_motion: Option<u8>,
    /// Bounce time (ms) of motion sensor input.
    pub pin_motion_bounce_ms: Option<ms>,
    /// BCM pin numbers of keypad row inputs.
    pub pins_keypad_rows: Option<[u8; 4]>,
    /// Bounce time (ms) of keypad row inputs.
    pub pins_keypad_rows_bounce_ms: Option<ms>
}

#[serde(rename_all = "kebab-case")]
#[derive(Deserialize, Copy, Clone, Debug)]
pub struct GpioOutputsConfig {
    /// BCM pin number of ringer output.
    pub pin_ringer: Option<u8>,
    /// BCM pin number of vibration motor output.
    pub pin_vibrate: Option<u8>,
    /// BCM pin numbers of keypad column outputs.
    pub pins_keypad_cols: Option<[u8; 3]>,
}

#[serde(rename_all = "kebab-case")]
#[derive(Deserialize, Copy, Clone, Debug)]
pub struct DebugConfig {
    /// Plays the panic tone when a Lua script encounters an error.
    pub enable_panic_tone: Option<bool>
}

pub fn load_config(path: &str) -> CursedConfig {
    let config_str = fs::read_to_string(path).expect("Unable to read config file");
    let config: CursedConfig = toml::from_str(&config_str).expect("Unable to parse config file");
    config
}