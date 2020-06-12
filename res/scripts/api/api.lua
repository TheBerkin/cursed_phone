--[[

    /==========================================================================\
    |========================= CURSED PHONE API FILE ==========================|
    |==========================================================================|
    | This script is required by phone services in order to function properly. |
    | Unless you are making changes to the engine, do not modify this file.    |
    \==========================================================================/
    
]]

--------------------
---- Native API ----
--------------------

NATIVE_API(function()
    --- Gets the number of seconds elapsed since the engine was initialized.
    --- @return number
    function get_run_time() end

    --- Gets the number of seconds elapsed since the current call started.
    --- Returns 0 if no call is active.
    --- @return number
    function get_call_time() end

    --- Pauses execution for the specified number of milliseconds.
    --- @param ms integer
    --- @type function
    function sleep(ms) end

    --- Generates a random integer between an inclusive minimum and exclusive maximum.
    --- @param min integer
    --- @param max integer
    --- @type function
    function rand_int(min, max) end

    --- Generates a random integer between an inclusive minimum and exclusive maximum,
    --- but avoids the `skip` value within the range.
    --- @param min integer
    --- @param skip integer
    --- @param max integer
    --- @type function
    function rand_int_skip(min, skip, max) end

    --- Generates a random integer between an inclusive minimum and exclusive maximum
    --- with an asymptotal distribution biased to lower values.
    --- @param min integer
    --- @param max integer
    --- @type function
    function rand_int_bias_low(min, max) end

    --- Generates a random integer between an inclusive minimum and exclusive maximum
    --- with an asymptotal distribution biased to higher values.
    --- @param min integer
    --- @param max integer
    --- @type function
    function rand_int_bias_high(min, max) end

    --- Generates a random floating-point number between an inclusive minimum and exclusive maximum.
    --- @param min number
    --- @param max number
    --- @type function
    function rand_float(min, max) end

    --- Returns a boolean value with probability `p` of being true, where `0.0 <= p <= 1.0`.
    --- @param p number
    --- @return boolean
    function chance(p) end
end)

--- Removes all keys from table `t`.
--- @param t table
function table.clear(t)
    for k in pairs(t) do
        rawset(t, k, nil)
    end
end

--- Returns a random element from `t`.
function table.random_choice(t)
    return t[rand_int(1, #t + 1)]
end

function is_number(val)
    return type(val) == 'number'
end

function is_string(val)
    return type(val) == 'string'
end