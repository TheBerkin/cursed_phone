--- @meta

--- @class SoundPlayOptions
--- @field volume number? @ Amplitude is multiplied by this value (Default: `1.0`)
--- @field interrupt boolean? @ Indicates whether to stop other sounds on the channel before playing (Default: `true`)
--- @field speed number? @ Speed multiplier for sound; affects both tempo and pitch (Default: `1.0`)
--- @field looping boolean? @ Indicates whether to make the sound loop forever (Default: `false`)
--- @field skip 'random' | number? @ Skip forward by `skip` seconds. Affected by `speed`. (Default: `0.0`)
--- @field take number? @ Cut sound to maximum of `take` seconds Affected by `speed`. (Default: `nil`)
--- @field delay number? @ Add `delay` seconds of silence before the sound. Not affected by `speed`. (Default: `nil`)
--- @field fadein number? @ Fades in the sound over `fadein` seconds. Not affected by `speed`. (Default: `0`)

--- Provides functions for controlling multi-channel sound playback.
--- @class SoundLib
sound = {}

--- Begins playing a sound on a specific channel.
--- @param path string @ A soundglob or path to the sound to play. Soundglobs will play a random matching sound.
--- @param channel Channel @ The channel to play the sound on.
--- @param opts SoundPlayOptions? @ The options to apply to the played sound.
--- @return boolean @ Indicates whether playback was successfully started.
--- @return number? @ The duration of the sound in seconds, if known and finite. Due to a limitation of the sound engine, only WAV sounds can currently report their length. 
function sound.play(path, channel, opts) end

--- Returns a boolean indicating whether the specified channel is playing something.
--- @param channel Channel
--- @return boolean
--- @nodiscard
function sound.is_busy(channel) end

--- Stops playback on a specific channel.
--- @param channel Channel
function sound.stop(channel) end

--- Stops playback on all channels.
function sound.stop_all() end

--- Gets the volume of the specified channel.
--- @param channel Channel
--- @return number
--- @nodiscard
function sound.get_channel_volume(channel) end

--- Gets the fade volume of the specified channel.
--- @param channel Channel
--- @return number
--- @nodiscard
function sound.get_channel_fade_volume(channel) end

--- Sets the fade volume of the specified channel.
--- @param channel Channel
--- @param volume number
function sound.set_channel_fade_volume(channel, volume) end

--- Gets the master volume.
--- @return number
--- @nodiscard
function sound.get_master_volume() end

--- Gets a boolean value indicating whether the specified sound channel is muted.
--- @param channel Channel @ The sound channel whose muted status to retrieve.
--- @return boolean
--- @nodiscard
function sound.is_channel_muted(channel) end

--- Sets the muted status of the specified sound channel.
--- @param channel Channel @ The sound channel whose muted status to change.
---@param muted boolean @ The muted status to set on the channel.
function sound.set_channel_muted(channel, muted) end

--- Sets the master volume.
--- @param volume number
function sound.set_master_volume(volume) end

--- Sets the volume of the specified channel.
--- @param channel Channel
--- @param volume number
function sound.set_channel_volume(channel, volume) end

--- @param channel Channel
--- @return number
function sound.get_channel_speed(channel) end

--- @param channel Channel
--- @param speed number
function sound.set_channel_speed(channel, speed) end

--- Plays a busy tone on `CHAN_SIGIN`.
function sound.play_busy_tone() end

--- Plays a fast busy tone on `CHAN_SIGIN`.
function sound.play_fast_busy_tone() end

--- Plays a ringback tone on `CHAN_SIGIN`.
function sound.play_ringback_tone() end

--- Plays a dial tone on `CHAN_SIGIN`.
function sound.play_dial_tone() end

--- Plays an off-hook tone on `CHAN_SIGIN`.
function sound.play_off_hook_tone() end

--- Plays a Special Information Tone (SIT) on `CHAN_SIGIN`.
--- @param sit_type SpecialInfoTone @ The type of SIT to play.
function sound.play_special_info_tone(sit_type) end

--- Plays the specified DTMF digit on `CHAN_SIGOUT`.
--- @param digit string
--- @param duration number
--- @param volume number
function sound.play_dtmf_digit(digit, duration, volume) end