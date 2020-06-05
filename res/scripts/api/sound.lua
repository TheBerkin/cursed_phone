--[[

    /==========================================================================\
    |========================= CURSED PHONE API FILE ==========================|
    |==========================================================================|
    | This script is required by phone services in order to function properly. |
    | Unless you are making changes to the engine, do not modify this file.    |
    \==========================================================================/
    
]]

-- ====================================================
-- ==================== SOUND API =====================
-- ====================================================

-- ========================
-- SOUND CHANNEL CONSTANTS
-- ========================

--- Channel for telephony signal tones
CHAN_TONE = 0
--- Phone channel 1
CHAN_PHONE1 = 1
--- Phone channel 2
CHAN_PHONE2 = 2
--- Phone channel 3
CHAN_PHONE3 = 3
--- Phone channel 4
CHAN_PHONE4 = 4
--- Phone channel 5
CHAN_PHONE5 = 5
--- Phone channel 6
CHAN_PHONE6 = 6
--- Phone channel 7
CHAN_PHONE7 = 7
--- Phone channel 8
CHAN_PHONE8 = 8
--- Soul channel 1
CHAN_SOUL1 = 9
--- Soul channel 2
CHAN_SOUL2 = 10
--- Soul channel 3
CHAN_SOUL3 = 11
--- Soul channel 4
CHAN_SOUL4 = 12
--- Background channel 1
CHAN_BG1 = 13
--- Background channel 2
CHAN_BG2 = 14
--- Background channel 3
CHAN_BG3 = 15
--- Background channel 4
CHAN_BG4 = 16

NATIVE_API(function()
    sound = {}

    --- Plays a sound on a specific channel.
    ---
    --- Available options:
    --- * `looping: boolean` Make the sound loop (Default: `false`)
    --- * `interrupt: boolean` Stop other sounds on the channel before playing (Default: `true`)
    --- * `speed: number` Multiply the playback speed (Default: `1.0`)
    --- * `volume: number` Multiply each sample by this value (Default: `1.0`)
    --- @param path string
    --- @param channel integer
    --- @param opts table
    function sound.play(path, channel, opts) end

    --- Returns a boolean indicating whether the specified channel is playing something.
    --- @param channel integer
    --- @return boolean
    function sound.is_busy(channel) end

    --- Stops playback on a specific channel.
    --- @param channel integer
    function sound.stop(channel) end

    --- Stops playback on all channels.
    function sound.stop_all() end

    --- Gets the volume of the specified channel.
    --- @param channel integer
    --- @return number
    function sound.get_channel_volume(channel) end

    --- Sets the volume of the specified channel.
    --- @param channel integer
    --- @param volume number
    function sound.set_channel_volume(channel, volume) end

    --- Loads the sound bank `bank_name` into memory, making it available for use.
    --- @return boolean
    function sound.load_bank(bank_name) end

    --- Unloads the sound bank `bank_name` from memory.
    --- @return boolean
    function sound.unload_bank(bank_name) end

    --- Returns a boolean value indicating whether the sound bank `bank_name` is currently loaded.
    --- @return boolean
    function sound.is_bank_loaded(bank_name) end
    
    --- Gets the master volume.
    --- @return number
    function sound.get_master_volume() end

    --- Sets the master volume.
    --- @param volume number
    function sound.set_master_volume(volume) end

    --- Plays a busy tone on `CHAN_TONE`.
    function sound.play_busy_tone() end

    --- Plays a fast busy tone on `CHAN_TONE`.
    function sound.play_fast_busy_tone() end

    --- Plays a ringback tone on `CHAN_TONE`.
    function sound.play_ringback_tone() end

    --- Plays a dial tone on `CHAN_TONE`.
    function sound.play_dial_tone() end

    --- Plays the specified DTMF digit.
    --- @param digit PhoneDigit
    --- @param duration number
    --- @param volume number
    function sound.play_dtmf_digit(digit, duration, volume) end
end)

--- *(Service use only)*
---
--- Plays a sound on a specific channel and waits asynchronously for it to end.
---
--- Available options:
--- * `looping: boolean` Make the sound loop (Default: `false`)
--- * `interrupt: boolean` Stop other sounds on the channel before playing (Default: `true`)
--- * `speed: number` Multiply the playback speed (Default: `1.0`)
--- * `volume: number` Multiply each sample by this value (Default: `1.0`)
--- @param path string
--- @param channel integer
--- @param opts table
function sound.play_wait(path, channel, opts)
    sound.play(path, channel, opts)
    while sound.is_busy(channel) do
        service.status(SERVICE_STATUS_WAITING)
    end
end

--- *(Service use only)*
---
--- Waits for the specified sound channel to finish playing.
function sound.wait(channel)
    while sound.is_busy(channel) do
        service.status(SERVICE_STATUS_WAITING)
    end
end

--- *(Service use only)*
---
--- Waits at least `duration` seconds for the specified sound channel to finish playing.
---
--- Keeps waiting even if `duration` lasts longer than the sound.
function sound.wait_min(channel, duration)
    local start_time = get_run_time();
    while sound.is_busy(channel) and get_run_time() - start_time < duration do
        service.status(SERVICE_STATUS_WAITING)
    end
end