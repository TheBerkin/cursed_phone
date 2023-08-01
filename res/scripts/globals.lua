--[[

    /==========================================================================\
    |========================= CURSED PHONE API FILE ==========================|
    |==========================================================================|
    | This script is required by the engine in order to function properly.     |
    | Unless you are making changes to the engine, do not modify this file.    |
    \==========================================================================/
    
]]

--- Alias for `coroutine.yield`
yield = coroutine.yield

function is_number(val)
    return type(val) == 'number'
end

function is_string(val)
    return type(val) == 'string'
end

function is_nan(val)
    return val ~= val and type(val) == 'number'
end

function is_some(val)
    return val ~= nil
end

--- @param val any
--- @return boolean
function coerce_boolean(val)
    return not not val
end

--- Converts decibels (dB) to an amplitude scale factor.
function scale_db(db)
    return math.pow(10.0, db / 20.0)
end