use std::fs;
use serde::Deserialize;
use toml;

#[allow(non_camel_case_types)]
type ms = u64;

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct CursedConfig {
    /// Number of times per second to update the phone state.
    /// Higher is better, but will also consume more CPU cycles.
    pub tick_rate: f64,

    /// Max number of instructions to allow per script execution
    pub script_execution_limit: Option<u32>,

    /// Direcories to load resources from
    pub include_resources: Vec<String>,

    /// **Post Dial Delay (PDD)**
    /// 
    /// Delay (in seconds) to wait after the last digit is dialed, 
    /// before the phone attempts to place the call.
    pub pdd: f32,

    /// Delay (in seconds) before off-hook intercept message is played.
    pub off_hook_delay: f32,
    
    /// Allows the host device to receive calls.
    pub allow_incoming_calls: Option<bool>,

    /// Enables ringer.
    pub ringer_enabled: Option<bool>,

    /// The default ring pattern expression assigned to agents.
    pub default_ring_pattern: String,

    /// Enables switchhook dialing.
    pub shd_enabled: Option<bool>,

    /// Maximum seconds between pulses for switch-hook dialing.
    pub shd_manual_pulse_interval: f32,

    /// Number of seconds phone must be on the hook to end the call.
    /// 
    /// **Must** be greater than manual_pulse_interval!
    /// 
    /// (Only used if `shd_enabled == true`)
    pub shd_hangup_delay: f32,

    /// Rotary dial configuration.
    #[serde(default)]
    pub rotary: RotaryDialConfig,

    /// Keypad configuration.
    #[serde(default)]
    pub keypad: KeypadConfig,

    /// Payphone configuration.
    #[serde(default)]
    pub payphone: PayphoneConfig,

    /// Sound configuration.
    pub sound: SoundConfig,

    /// GPIO configuration.
    pub gpio: GpioConfig,

    /// Debug feature configuration.
    pub debug: Option<DebugConfig>
}

#[derive(Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "kebab-case", default)]
pub struct RotaryDialConfig {
    pub enabled: bool,

    /// Describes the digit mapping for pulse dialing, sorted by pulse count.
    pub digit_layout: String,

    /// Delay (in milliseconds) between dial leaving resting state and first valid pulse.
    pub first_pulse_delay_ms: Option<ms>,

    /// Input configuration for the dial (pulse component).
    pub input_pulse: Option<InputPinConfig>,

    /// Input configuration for the dial (rest component).
    pub input_rest: Option<InputPinConfig>,
}

#[derive(Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "kebab-case", default)]
pub struct KeypadConfig {
    pub enabled: bool,

    /// BCM pin numbers of keypad row inputs.
    pub input_rows: Option<[u8; 4]>,

    /// BCM pin numbers of keypad column outputs.
    pub output_cols: Option<[u8; 3]>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case", default)]
pub struct PayphoneConfig {
    pub enabled: bool,

    /// Monetary value constants for coin triggers.
    /// Set values in terms of the smallest unit of your currency.
    pub coin_values: Option<Vec<u32>>,

    /// Input pins for the coin trigger switches.
    /// Must be the same length as `coin_values`.
    pub coin_input_pins: Option<Vec<u8>>,

    /// Bounce times for the coin trigger switch pins.
    /// Must be the same length as `coin_values`.
    pub coin_input_bounce_ms: Option<Vec<ms>>,

    /// Pull type for the coin trigger switch pins.
    pub coin_input_pull: Option<String>,

    /// The default rate (in cents) applied to calls.
    ///
    /// Agents may opt to override this with their own rate by using the `AgentModule.set_custom_price()` Lua method:
    /// ```lua
    /// local S = AGENT_MODULE("operator", "0")
    /// S:set_custom_price(0) -- Make calls to this number free
    /// ```
    pub standard_call_rate: u32,

    /// The delay (in milliseconds) between an outgoing call being accepted and the coin deposit being consumed.
    /// Defaults to 0 (instant).
    pub coin_consume_delay_ms: ms,

    /// Amount of call time (as whole seconds) credited for the standard (or custom) call rate.
    /// Defaults to 0 (no time limit).
    pub time_credit_seconds: u64,

    /// Amount of call time remaining (as whole seconds) before the Tollmaster alerts the user.
    pub time_credit_warn_seconds: u64,

    /// Allows agents to set their own prices. (default: `true`)
    pub enable_custom_agent_rates: bool,
}

impl Default for PayphoneConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            coin_input_pins: None,
            coin_input_bounce_ms: None,
            coin_input_pull: None,
            coin_values: None,
            standard_call_rate: 0,
            coin_consume_delay_ms: 0,
            time_credit_seconds: 0,
            time_credit_warn_seconds: 60,
            enable_custom_agent_rates: true,
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct SoundConfig {
    /// Initial master volume.
    pub master_volume: f32,
    pub dtmf_volume: f32,
    pub dtmf_tone_duration_ms: ms,
    pub dial_tone_gain: f32,
    pub ringback_tone_gain: f32,
    pub busy_tone_gain: f32,
    pub off_hook_tone_gain: f32,
    pub special_info_tone_gain: f32,
    pub comfort_noise_name: Option<String>,
    pub comfort_noise_volume: f32
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct GpioConfig {
    /// Input configuration.
    pub inputs: GpioInputsConfig,
    /// Output configuration.
    pub outputs: GpioOutputsConfig
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct GpioInputsConfig {
    /// Input configuration for the switchhook.
    pub switchhook: InputPinConfig,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct InputPinConfig {
    /// BCM pin number of the input.
    pub pin: u8,
    /// Bounce time (ms) of the input.
    pub bounce_ms: Option<ms>,
    /// Name of the resistor type to use. Defaults to "none".
    pub pull: Option<String>
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct GpioOutputsConfig {
    /// BCM pin number of ringer output.
    pub pin_ringer: Option<u8>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct DebugConfig {
    /// Plays the panic tone when a Lua script encounters an error.
    pub enable_panic_tone: Option<bool>
}

pub fn load_config(path: &str) -> CursedConfig {
    let config_str = fs::read_to_string(path).expect("Unable to read config file");
    let config: CursedConfig = toml::from_str(&config_str).expect("Unable to parse config file");
    config
}