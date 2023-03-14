--[[

    /==========================================================================\
    |========================= CURSED PHONE API FILE ==========================|
    |==========================================================================|
    | This script is required by the engine in order to function properly.     |
    | Unless you are making changes to the engine, do not modify this file.    |
    \==========================================================================/
    
]]

--- @class TimeSince
--- @field start_time number
--- @field elapsed number

local GETTERS_TimeSince = {
    --- @param self TimeSince
    elapsed = function(self)
        return engine_time() - self.start_time
    end
}

local META_TimeSince = {
    __index = function(self, i)
        local prop = rawget(GETTERS_TimeSince, i)
        return prop and prop(self) or nil
    end
}

--- Creates a timer that measures how much time has elapsed since a specific point in time.
--- @param time number? @ The time (since engine epoch) to measure from. Exclude to use the current time.
--- @return TimeSince
function time_since(time)
    local t = {
        start_time = engine_time()
    }
    setmetatable(t, META_TimeSince)
    return t
end

--- @class TimeUntil
--- @field target_time number
--- @field passed boolean
--- @field remaining number

local GETTERS_TimeUntil = {
    passed = function(self)
        return engine_time() > self.target_time
    end,
    remaining = function(self)
        return math.max(0, self.target_time - engine_time())
    end
}

local META_TimeUntil = {
    __index = function(self, i)
        local prop = rawget(GETTERS_TimeUntil, i)
        return prop and prop(self) or nil
    end
}

--- Creates a timer that measures how much time remains until a specific future point in time.
--- @param delta number @ The number of seconds in the future to set the timer.
--- @return TimeUntil
function time_until(delta)
    local t = {
        target_time = engine_time() + delta
    }
    setmetatable(t, META_TimeUntil)
    return t
end