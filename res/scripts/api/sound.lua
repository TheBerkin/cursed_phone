--[[

    /==========================================================================\
    |========================= CURSED PHONE API FILE ==========================|
    |==========================================================================|
    | This script is required by the engine in order to function properly.     |
    | Unless you are making changes to the engine, do not modify this file.    |
    \==========================================================================/
    
]]

-- ====================================================
-- ==================== SOUND API =====================
-- ====================================================


--- @alias SoundChannel integer

--- Channel for incoming telephony signal tones.
--- @type SoundChannel
CHAN_SIGIN = 0
--- Channel for incoming comfort noise.
--- @type SoundChannel
CHAN_NOISEIN = 1
--- Channel for outgoing telephony signal tones.
--- @type SoundChannel
CHAN_SIGOUT = 2
--- Phone Channel 1
--- @type SoundChannel
CHAN_PHONE1 = 3
--- Phone Channel 2
--- @type SoundChannel
CHAN_PHONE2 = 4
--- Phone Channel 3
--- @type SoundChannel
CHAN_PHONE3 = 5
--- Phone Channel 4
--- @type SoundChannel
CHAN_PHONE4 = 6
--- Phone Channel 5
--- @type SoundChannel
CHAN_PHONE5 = 7
--- Phone Channel 6
--- @type SoundChannel
CHAN_PHONE6 = 8
--- Phone Channel 7
--- @type SoundChannel
CHAN_PHONE7 = 9
--- Phone Channel 8
--- @type SoundChannel
CHAN_PHONE8 = 10
--- Soul Channel 1
--- @type SoundChannel
CHAN_SOUL1 = 11
--- Soul Channel 2
--- @type SoundChannel
CHAN_SOUL2 = 12
--- Soul Channel 3
--- @type SoundChannel
CHAN_SOUL3 = 13
--- Soul Channel 4
--- @type SoundChannel
CHAN_SOUL4 = 14
--- Background Channel 1
--- @type SoundChannel
CHAN_BG1 = 15
--- Background Channel 2
--- @type SoundChannel
CHAN_BG2 = 16
--- Background Channel 3
--- @type SoundChannel
CHAN_BG3 = 17
--- Background Channel 4
--- @type SoundChannel
CHAN_BG4 = 18


--- @class SoundPlayOptions
--- @field volume number? @ Amplitude is multiplied by this value (Default: `1.0`)
--- @field interrupt boolean? @ Indicates whether to stops other sounds on the channel before playing (Default: `true`)
--- @field speed number? @ Speed multiplier for sound; affects both tempo and pitch (Default: `1.0`)
--- @field looping boolean? @ Indicates whether to make the sound loop forever (Default: `false`)
--- @field skip number? @ Skip forward by `skip` seconds (before speed adjustment) (Default: `0.0`)
--- @field take number? @ Cut sound to maximum of `take` seconds (before speed adjustment) (Default: `nil`)
--- @field fadein number? @ Fades in the sound over `fadein` seconds (after speed adjustment) (Default: `0`)

if not sound then
    --- Provides functions for controlling multi-channel sound playback.
    sound = {}

    --- Plays a sound on a specific channel.
    --- @param path string @ A soundglob or path to the sound to play. Soundglobs will play a random matching sound.
    --- @param channel SoundChannel @ The channel to play the sound on.
    --- @param opts SoundPlayOptions? @ The options to apply to the played sound.
    function sound.play(path, channel, opts) end

    --- Returns a boolean indicating whether the specified channel is playing something.
    --- @param channel SoundChannel
    --- @return boolean
    function sound.is_busy(channel) return false end

    --- Stops playback on a specific channel.
    --- @param channel SoundChannel
    function sound.stop(channel) end

    --- Stops playback on all channels.
    function sound.stop_all() end

    --- Gets the volume of the specified channel.
    --- @param channel SoundChannel
    --- @return number
    function sound.get_channel_volume(channel) return 0 end

    --- Gets the fade volume of the specified channel.
    --- @param channel SoundChannel
    --- @return number
    function sound.get_channel_fade_volume(channel) return 0 end

    --- Sets the fade volume of the specified channel.
    --- @param channel SoundChannel
    --- @param volume number
    function sound.set_channel_fade_volume(channel, volume) end

    --- Gets the master volume.
    --- @return number
    function sound.get_master_volume() return 0 end

    --- Gets a boolean value indicating whether the specified sound channel is muted.
    --- @param channel SoundChannel @ The sound channel whose muted status to retrieve.
    --- @return boolean
    function sound.is_channel_muted(channel) return false end

    --- Sets the muted status of the specified sound channel.
    --- @param channel SoundChannel @ The sound channel whose muted status to change.
    ---@param muted boolean @ The muted status to set on the channel.
    function sound.set_channel_muted(channel, muted) end

    --- Sets the master volume.
    --- @param volume number
    function sound.set_master_volume(volume) end

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
    --- @param digit PhoneDigit
    --- @param duration number
    --- @param volume number
    function sound.play_dtmf_digit(digit, duration, volume) end
end

--- *(Agent use only)*
---
--- Plays a sound on a specific channel and waits asynchronously for it to end.
--- @param path string
--- @param channel integer
--- @param opts SoundPlayOptions?
function sound.play_wait(path, channel, opts)
    sound.play(path, channel, opts)
    while sound.is_busy(channel) do
        agent.intent(AGENT_INTENT_WAIT)
    end
end

--- *(Agent use only)*
---
--- Plays a sound on a specific channel and waits asynchronously for it to end or until the specified predicate returns true.
---
--- Additional options:
--- * `early_stop: boolean` Stop the channel if canceled (Default: `true`)
--- @param path string
--- @param channel integer
--- @param predicate function
--- @param opts SoundPlayOptions?
function sound.play_wait_cancel(path, channel, predicate, opts)
    if not predicate or predicate() then return end
    sound.play(path, channel, opts)
    while not predicate() and sound.is_busy(channel) do
        agent.intent(AGENT_INTENT_WAIT)
    end
    if not opts or opts.early_stop == nil or opts.early_stop == true then
        sound.stop(channel)
    end
end

--- *(Agent use only)*
---
--- Fades out the sound on the specified channel over `duration` seconds, then stops the sound. 
--- @param channel SoundChannel @ The channel to fade out
--- @param duration number @ The duration of the fade in seconds
function sound.fade_out(channel, duration)
    if not sound.is_busy(channel) then return end
    local fade_volume_start = sound.get_channel_fade_volume(channel)
    local start_time = engine_time()
    local end_time = engine_time() + duration
    while true do
        local time = engine_time()
        local progress = math.invlerp(time, start_time, end_time, true)

        if not sound.is_busy(channel) then return end

        if progress >= 1 then
            sound.stop(channel)
            return
        else
            local fade_volume = math.lerp(1, 0, progress, true)
            sound.set_channel_fade_volume(channel, fade_volume)
        end
        agent.intent(AGENT_INTENT_WAIT)
    end
end

--- *(Agent use only)*
---
--- Waits for the specified sound channel to finish playing.
function sound.wait(channel)
    while sound.is_busy(channel) do
        agent.intent(AGENT_INTENT_WAIT)
    end
end

--- *(Agent use only)*
---
--- Waits at least `duration` seconds for the specified sound channel to finish playing.
---
--- Keeps waiting even if `duration` lasts longer than the sound.
function sound.wait_min(channel, duration)
    local start_time = engine_time();
    while sound.is_busy(channel) or engine_time() - start_time < duration do
        agent.intent(AGENT_INTENT_WAIT)
    end
end

--- *(Agent use only)*
---
--- Waits at most `duration` seconds for the specified sound channel to finish playing.
---
--- If the sound stops within `duration`, the wait is canceled.
function sound.wait_max(channel, duration)
    local start_time = engine_time();
    while sound.is_busy(channel) and engine_time() - start_time < duration do
        agent.intent(AGENT_INTENT_WAIT)
    end
end