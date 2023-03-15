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


--- @enum Channel
--- Defines available sound playback channels. They come in a few flavors:
--- - `PHONE*`: Phone channels are for call audio (excluding telephony signals).
--- - `SOUL*`: Soul channels are for souls to speak freely outside of calls. Hanging up a call will not silence these channels.
--- - `BG*`: Background channels are for miscellaneous use.
Channel = {
    --- Incoming signal channel (e.g. dial tone, SITs, busy signal...)
    SIG_IN = 0,
    --- Comfort noise channel
    NOISE_IN = 1,
    --- Outgoing signal channel (DTMF)
    SIG_OUT = 2,
    --- Phone Channel 1
    PHONE01 = 3,
    --- Phone Channel 2
    PHONE02 = 4,
    --- Phone Channel 3
    PHONE03 = 5,
    --- Phone Channel 4
    PHONE04 = 6,
    --- Phone Channel 5
    PHONE05 = 7,
    --- Phone Channel 6
    PHONE06 = 8,
    --- Phone Channel 7
    PHONE07 = 9,
    --- Phone Channel 8
    PHONE08 = 10,
    --- Phone Channel 9
    PHONE09 = 11,
    --- Phone Channel 10
    PHONE10 = 12,
    --- Soul Channel 1
    SOUL01 = 13,
    --- Soul Channel 2
    SOUL02 = 14,
    --- Soul Channel 3
    SOUL03 = 15,
    --- Soul Channel 4
    SOUL04 = 16,
    --- Background Channel 1
    BG01 = 17,
    --- Background Channel 2
    BG02 = 18,
    --- Background Channel 3
    BG03 = 19,
    --- Background Channel 4
    BG04 = 20,
    --- Debug Channel
    DEBUG = 21,
}

--- All `PHONE*` sound channels.
ALL_PHONE_CHANNELS = { Channel.PHONE01, Channel.PHONE02, Channel.PHONE03, Channel.PHONE04, Channel.PHONE05, Channel.PHONE06, Channel.PHONE07, Channel.PHONE08, Channel.Phone09, Channel.Phone10 }

--- All `SOUL*` sound channels.
ALL_SOUL_CHANNELS = { Channel.SOUL01, Channel.SOUL02, Channel.SOUL03, Channel.SOUL04 }

--- All `BG*` sound channels.
ALL_BG_CHANNELS = { Channel.BG01, Channel.BG02, Channel.BG03, Channel.BG04 }


--- @class SoundPlayOptions
--- @field volume number? @ Amplitude is multiplied by this value (Default: `1.0`)
--- @field interrupt boolean? @ Indicates whether to stop other sounds on the channel before playing (Default: `true`)
--- @field speed number? @ Speed multiplier for sound; affects both tempo and pitch (Default: `1.0`)
--- @field looping boolean? @ Indicates whether to make the sound loop forever (Default: `false`)
--- @field skip 'random' | number? @ Skip forward by `skip` seconds. Affected by `speed`. (Default: `0.0`)
--- @field take number? @ Cut sound to maximum of `take` seconds Affected by `speed`. (Default: `nil`)
--- @field delay number? @ Add `delay` seconds of silence before the sound. Not affected by `speed`. (Default: `nil`)
--- @field fadein number? @ Fades in the sound over `fadein` seconds. Not affected by `speed`. (Default: `0`)

if not sound then
    --- Provides functions for controlling multi-channel sound playback.
    --- @class SoundLib
    sound = {}

    --- Begins playing a sound on a specific channel.
    --- @param path string @ A soundglob or path to the sound to play. Soundglobs will play a random matching sound.
    --- @param channel Channel @ The channel to play the sound on.
    --- @param opts SoundPlayOptions? @ The options to apply to the played sound.
    --- @return boolean @ Indicates whether playback was successfully started.
    --- @return number? @ The duration of the sound in seconds, if known and finite. Due to a limitation of the sound engine, only WAV sounds can currently report their length. 
    function sound.play(path, channel, opts) return false, nil end

    --- Returns a boolean indicating whether the specified channel is playing something.
    --- @param channel Channel
    --- @return boolean
    --- @nodiscard
    function sound.is_busy(channel) return false end

    --- Stops playback on a specific channel.
    --- @param channel Channel
    function sound.stop(channel) end

    --- Stops playback on all channels.
    function sound.stop_all() end

    --- Gets the volume of the specified channel.
    --- @param channel Channel
    --- @return number
    --- @nodiscard
    function sound.get_channel_volume(channel) return 0 end

    --- Gets the fade volume of the specified channel.
    --- @param channel Channel
    --- @return number
    --- @nodiscard
    function sound.get_channel_fade_volume(channel) return 0 end

    --- Sets the fade volume of the specified channel.
    --- @param channel Channel
    --- @param volume number
    function sound.set_channel_fade_volume(channel, volume) end

    --- Gets the master volume.
    --- @return number
    --- @nodiscard
    function sound.get_master_volume() return 0 end

    --- Gets a boolean value indicating whether the specified sound channel is muted.
    --- @param channel Channel @ The sound channel whose muted status to retrieve.
    --- @return boolean
    --- @nodiscard
    function sound.is_channel_muted(channel) return false end

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

--- @async
--- *(Agent use only)*
---
--- Plays a sound on a specific channel and waits asynchronously for it to end.
--- @param path string
--- @param channel Channel
--- @param opts SoundPlayOptions?
--- @param wait_time_offset number?
function sound.play_wait(path, channel, opts, wait_time_offset)
    local success, duration = sound.play(path, channel, opts)
    if wait_time_offset and duration then
        agent.wait(duration + wait_time_offset)
    else
        while sound.is_busy(channel) do
            agent.intent(IntentCode.WAIT)
        end
    end
end

--- @class SoundPlayWaitCancelOptions: SoundPlayOptions
--- @field early_stop boolean

--- @async
--- *(Agent use only)*
---
--- Plays a sound on a specific channel and waits asynchronously for it to end or until the specified predicate returns true.
---
--- Additional options:
--- * `early_stop: boolean` Stop the channel if canceled (Default: `true`)
--- @param path string
--- @param channel Channel
--- @param predicate function
--- @param opts SoundPlayWaitCancelOptions?
function sound.play_wait_cancel(path, channel, predicate, opts)
    if not predicate or predicate() then return end
    sound.play(path, channel, opts)
    while not predicate() and sound.is_busy(channel) do
        agent.intent(IntentCode.WAIT)
    end
    if not opts or opts.early_stop == nil or opts.early_stop == true then
        sound.stop(channel)
    end
end

--- @async
--- *(Agent use only)*
---
--- Fades out the sound on the specified channel over `duration` seconds, then stops the sound. 
--- @param channels Channel | Channel[] @ The channel to fade out
--- @param duration number @ The duration of the fade in seconds
function sound.fade_out(channels, duration)
    local channel_type = type(channels)
    local start_time = engine_time()
    local end_time = engine_time() + duration

    if channel_type == 'number' or channel_type == 'integer' then
        if not sound.is_busy(channels) then return end
        while true do
            local time = engine_time()
            local progress = math.invlerp(time, start_time, end_time, true)
    
            if not sound.is_busy(channels) then return end
    
            if progress >= 1 then
                sound.stop(channels)
                sound.set_channel_fade_volume(channels, 1)
                return
            else
                local fade_volume = math.lerp(1, 0, progress, true)
                sound.set_channel_fade_volume(channels, fade_volume)
            end
            agent.intent(IntentCode.WAIT)
        end    
    elseif channel_type == 'table' then
        while true do
            local time = engine_time()
            local progress = math.invlerp(time, start_time, end_time, true)
    
            if progress >= 1 then
                for i = 1, #channels do
                    local channel = channels[i]
                    sound.stop(channel)
                end
                return
            else
                local fade_volume = math.lerp(1, 0, progress, true)
                local is_any_channel_busy = false
                for i = 1, #channels do
                    local channel = channels[i]
                    if sound.is_busy(channel) then 
                        is_any_channel_busy = true
                        sound.set_channel_fade_volume(channel, fade_volume)
                    end
                end
    
                if not is_any_channel_busy then return end
            end
            agent.intent(IntentCode.WAIT)
        end
    else
        error("Input channels must be either integer or table", 2)
    end
end

--- @async
--- *(Agent use only)*
---
--- Waits for the specified sound channel to finish playing.
--- @param channel Channel @ The channel to wait for.
function sound.wait(channel)
    while sound.is_busy(channel) do
        agent.intent(IntentCode.WAIT)
    end
end

--- @async
--- *(Agent use only)*
---
--- Waits at least `duration` seconds for the specified sound channel to finish playing.
---
--- Keeps waiting even if `duration` lasts longer than the sound.
--- @param channel Channel @ The channel to wait for.
--- @param duration number @ The minimum number of seconds to wait for.
function sound.wait_min(channel, duration)
    local start_time = engine_time();
    while sound.is_busy(channel) or engine_time() - start_time < duration do
        agent.intent(IntentCode.WAIT)
    end
end

--- @async
--- *(Agent use only)*
---
--- Waits at most `duration` seconds for the specified sound channel to finish playing.
---
--- If the sound stops within `duration`, the wait is canceled.
--- @param channel Channel @ The channel to wait for.
--- @param duration number @ The maximum number of seconds to wait for.
function sound.wait_max(channel, duration)
    local start_time = engine_time();
    while sound.is_busy(channel) and engine_time() - start_time < duration do
        agent.intent(IntentCode.WAIT)
    end
end