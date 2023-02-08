--[[

    /==========================================================================\
    |========================= CURSED PHONE API FILE ==========================|
    |==========================================================================|
    | This script is required by the engine in order to function properly.     |
    | Unless you are making changes to the engine, do not modify this file.    |
    \==========================================================================/
    
]]

--------------------
---- Native API ----
--------------------

if _NATIVE_STUB then
    --- Gets the number of seconds elapsed since the engine was initialized.
    --- @return number
    function engine_time() return 0 end

    --- Gets the number of seconds elapsed since the current call started.
    --- Returns 0 if no call is active.
    --- @return number
    function call_time() return 0 end

    --- Pauses execution for the specified number of milliseconds.
    --- @param ms integer
    --- @type function
    function sleep(ms) end

    --- Generates a random integer between an inclusive minimum and exclusive maximum.
    --- @param min integer
    --- @param max integer
    --- @return integer
    function rand_int(min, max) return 0 end

    --- Generates a random integer between an inclusive minimum and exclusive maximum,
    --- but avoids the `skip` value within the range.
    --- @param min integer
    --- @param skip integer
    --- @param max integer
    --- @return integer
    function rand_int_skip(min, skip, max) return 0 end

    --- Generates a random integer between an inclusive minimum and exclusive maximum
    --- with an asymptotal distribution biased to lower values.
    --- @param min integer
    --- @param max integer
    --- @return integer
    function rand_int_bias_low(min, max) return 0 end

    --- Generates a random integer between an inclusive minimum and exclusive maximum
    --- with an asymptotal distribution biased to higher values.
    --- @param min integer
    --- @param max integer
    --- @return integer
    function rand_int_bias_high(min, max) return 0 end

    --- Generates a random floating-point number between an inclusive minimum and exclusive maximum.
    --- @param min number
    --- @param max number
    --- @return number
    function rand_float(min, max) return 0 end

    --- Returns a boolean value with probability `p` of being true, where `0.0 <= p <= 1.0`.
    --- @param p number
    --- @return boolean
    function chance(p) return false end
end

function is_number(val)
    return type(val) == 'number'
end

function is_string(val)
    return type(val) == 'string'
end