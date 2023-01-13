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
    /// |`"payphone"` |Payphone (keypad dial)   |
    /// |`"other"`    |Other/unknown phone type |
    pub phone_type: String,

    /// Number of times per second to update the phone state.
    /// Higher is better, but will also consume more CPU cycles.
    pub tick_rate: f64,

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

    /// Delay (in milliseconds) between dial leaving resting state and first valid pulse.
    pub rotary_first_pulse_delay_ms: Option<ms>,

    /// Payphone configuration.
    #[serde(default)]
    pub payphone: PayphoneConfig,

    /// Optional features configuration.
    pub features: FeaturesConfig,

    /// Sound configuration.
    pub sound: SoundConfig,

    /// GPIO configuration.
    pub gpio: GpioConfig,

    /// Debug feature configuration.
    pub debug: Option<DebugConfig>
}

#[serde(rename_all = "kebab-case", default)]
#[derive(Deserialize, Clone, Debug)]
pub struct PayphoneConfig {
    /// Monetary value constants for coin triggers.
    /// Set values in terms of the smallest unit of your currency.
    pub coin_values: Option<Vec<u32>>,

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
            coin_values: None,
            standard_call_rate: 0,
            coin_consume_delay_ms: 0,
            time_credit_seconds: 0,
            time_credit_warn_seconds: 60,
            enable_custom_agent_rates: true,
        }
    }
}

#[serde(rename_all = "kebab-case")]
#[derive(Deserialize, Clone, Debug)]
pub struct FeaturesConfig {
    /// Enables ringer.
    pub enable_ringer: Option<bool>,

    /// Enables switch-hook dialing.
    pub enable_switch_hook_dialing: Option<bool>,

    /// Allows the host device to receive calls.
    pub enable_incoming_calls: Option<bool>,
}

#[serde(rename_all = "kebab-case")]
#[derive(Deserialize, Clone, Debug)]
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
    pub comfort_noise_name: String,
    pub comfort_noise_volume: f32
}

#[serde(rename_all = "kebab-case")]
#[derive(Deserialize, Clone, Debug)]
pub struct GpioConfig {
    /// Input configuration.
    pub inputs: GpioInputsConfig,
    /// Output configuration.
    pub outputs: GpioOutputsConfig
}

#[serde(rename_all = "kebab-case")]
#[derive(Deserialize, Clone, Debug)]
pub struct GpioInputsConfig {
    /// Input configuration for the switchhook.
    pub hook: InputPinConfig,
    /// Input configuration for the dial (pulse component).
    pub dial_pulse: Option<InputPinConfig>,
    /// Input configuration for the dial (switch component).
    pub dial_switch: Option<InputPinConfig>,
    /// Input configuration for the motion sensor.
    pub motion: Option<InputPinConfig>,
    /// BCM pin numbers of keypad row inputs.
    pub keypad_row_pins: Option<[u8; 4]>,
    /// Input pins for the coin trigger switches.
    /// Must be the same length as `coin_values`.
    pub coin_trigger_pins: Option<Vec<u8>>,
    /// Bounce times for the coin trigger switch pins.
    /// Must be the same length as `coin_values`.
    pub coin_trigger_bounce_ms: Option<Vec<ms>>,
    /// Pull type for the coin trigger switch pins.
    pub coin_trigger_pull: Option<String>
}

#[serde(rename_all = "kebab-case")]
#[derive(Deserialize, Clone, Debug)]
pub struct InputPinConfig {
    /// BCM pin number of the input.
    pub pin: u8,
    /// Bounce time (ms) of the input.
    pub bounce_ms: Option<ms>,
    /// Name of the resistor type to use. Defaults to "none".
    pub pull: Option<String>
}

#[serde(rename_all = "kebab-case")]
#[derive(Deserialize, Clone, Debug)]
pub struct GpioOutputsConfig {
    /// BCM pin number of ringer output.
    pub pin_ringer: Option<u8>,
    /// BCM pin numbers of keypad column outputs.
    pub pins_keypad_cols: Option<[u8; 3]>,
}

#[serde(rename_all = "kebab-case")]
#[derive(Deserialize, Clone, Debug)]
pub struct DebugConfig {
    /// Plays the panic tone when a Lua script encounters an error.
    pub enable_panic_tone: Option<bool>
}

pub fn load_config(path: &str) -> CursedConfig {
    let config_str = fs::read_to_string(path).expect("Unable to read config file");
    let config: CursedConfig = toml::from_str(&config_str).expect("Unable to parse config file");
    config
}