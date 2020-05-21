phone = phone or {
    --- Plays a sound on a specific channel.
    --- @param path string
    --- @param channel integer
    --- @return integer
    play_sound = function(path, channel) end,
    --- Stops playback on a specific channel.
    --- @param channel integer
    stop_sound = function(channel) end
}

--- Pauses execution for the specified number of milliseconds.
sleep = sleep or function(ms) end