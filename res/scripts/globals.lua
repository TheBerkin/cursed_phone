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

    --- Calculates a 2-dimensional Perlin noise sample corresponding to the specified coordinates and noise parameters.
    --- @param x number @ The X coordinate of the noise sample.
    --- @param y number @ The Y coordinate of the noise sample.
    --- @param octaves integer @ The number of octaves (layers) to add to the noise.
    --- @param frequency number @ The number of noise cycles per unit length.
    --- @param persistence number @ The amplitude multiplier for each successive octave.
    --- @param lacunarity number @ The frequency multiplier for each successive octave.
    --- @param seed integer @ The seed of the noise generator.
    --- @return number
    function perlin_sample(x, y, octaves, frequency, persistence, lacunarity, seed) return 0 end

    --- @param agent_id integer
    --- @param loaded boolean
    function set_agent_sounds_loaded(agent_id, loaded) end
end

function is_number(val)
    return type(val) == 'number'
end

function is_string(val)
    return type(val) == 'string'
end

function rand_seed_32()
    return rand_int(-2147483648, 2147483648)
end