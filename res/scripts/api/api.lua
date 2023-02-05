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
    function chance(p) return false end
end

--- Removes all keys from table `t`.
--- @param t table
function table.clear(t)
    for k in pairs(t) do
        rawset(t, k, nil)
    end
end

--- Runs a function on every element of `t` and returns another table containing the results with the same keys.
--- @param t table
--- @param map_func fun(x: any): any
function table.map(t, map_func)
    local results = {}
    for k,v in pairs(t) do
        results[k] = map_func(v)
    end
    return results
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

--- Clamps `x` between `min` and `max`.
--- @param x number @ The number to clamp.
--- @param a number @ The first bound.
--- @param b number @ The second bound.
--- @return number
function math.clamp(x, a, b)
    local min = math.min(a, b)
    local max = math.max(a, b)
    if x < min then return min end
    if x > max then return max end
    return x
end

--- Linearly interpolates between `a` and `b`.
--- @param a number @ The startung value.
--- @param b number @ The ending value.
--- @param delta number @ The point between `a` and `b` to sample.
--- @param clamp boolean? @ Indicates whether to clamp `delta` between 0 and 1.  
--- @return number
function math.lerp(a, b, delta, clamp)
    if clamp then delta = math.clamp(delta, 0, 1) end 
    return (b - a) * delta + a
end

--- Performs an inverse linear interpolation on `x` between the bounds `a` and `b`.
--- @param x number @ The input value.
--- @param a number @ The starting bound.
--- @param b number @ The ending bound.
--- @param clamp boolean? @ Indicates whether to clamp the return value between 0 and 1.  
function math.invlerp(x, a, b, clamp)
    if clamp then
        return math.clamp((x - a) / (b - a), 0, 1)
    else
        return (x - a) / (b - a)
    end
end