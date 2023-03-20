--[[

    /==========================================================================\
    |========================= CURSED PHONE API FILE ==========================|
    |==========================================================================|
    | This script is required by the engine in order to function properly.     |
    | Unless you are making changes to the engine, do not modify this file.    |
    \==========================================================================/
    
]]

--- @class TimeSince
--- @field private _start_time number
TimeSince = {}

local M_TimeSince = {
    __index = TimeSince
}

--- @return number
function TimeSince:elapsed()
    return engine_time() - self._start_time
end

function TimeSince:reset()
    self._start_time = engine_time()
end

--- Creates a timer that measures how much time has elapsed since a specific point in time.
--- @param time number? @ The time (since engine epoch) to measure from. Exclude to use the current time.
--- @return TimeSince
function time_since(time)
    --- @type TimeSince
    local t = {
        _start_time = time or engine_time()
    }
    setmetatable(t, M_TimeSince)
    return t
end