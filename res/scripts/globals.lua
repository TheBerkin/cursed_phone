--[[

    /==========================================================================\
    |========================= CURSED PHONE API FILE ==========================|
    |==========================================================================|
    | This script is required by the engine in order to function properly.     |
    | Unless you are making changes to the engine, do not modify this file.    |
    \==========================================================================/
    
]]

local function stub() error("native binding is missing for this function", 3) end

--------------------
---- Native API ----
--------------------

if _NATIVE_STUB then
    --- Gets the number of seconds elapsed since the engine was initialized.
    --- @return number
    --- @diagnostic disable-next-line: missing-return
    function engine_time() stub() end

    --- Gets the number of seconds elapsed since the current call started.
    --- Returns 0 if no call is active.
    --- @return number
    --- @diagnostic disable-next-line: missing-return
    function call_time() stub() end

    --- Generates a random integer between an inclusive minimum and exclusive maximum.
    --- @param min integer
    --- @param max integer
    --- @return integer
    --- @diagnostic disable-next-line: missing-return
    function rand_int(min, max) stub() end

    --- Generates a random integer between an inclusive minimum and maximum.
    --- @param min integer
    --- @param max integer
    --- @return integer
    --- @diagnostic disable-next-line: missing-return
    function rand_int_i(min, max) stub() end

    --- Generates a random integer between an inclusive minimum and exclusive maximum,
    --- but avoids the `skip` value within the range.
    --- @param min integer
    --- @param skip integer
    --- @param max integer
    --- @return integer
    --- @diagnostic disable-next-line: missing-return
    function rand_int_skip(min, skip, max) stub() end

    --- Generates a random integer between an inclusive minimum and exclusive maximum
    --- with an asymptotal distribution biased to lower values.
    --- @param min integer
    --- @param max integer
    --- @return integer
    --- @diagnostic disable-next-line: missing-return
    function rand_int_bias_low(min, max) stub() end

    --- Generates a random integer between an inclusive minimum and exclusive maximum
    --- with an asymptotal distribution biased to higher values.
    --- @param min integer
    --- @param max integer
    --- @return integer
    --- @diagnostic disable-next-line: missing-return
    function rand_int_bias_high(min, max) stub() end

    --- Generates a random integer between an inclusive minimum and exclusive maximum, approximating a normal distribution.
    --- @param min integer
    --- @param max integer
    --- @return integer
    --- @diagnostic disable-next-line: missing-return
    function rand_int_normal(min, max) stub() end

    --- Generates a random floating-point number between an inclusive minimum and exclusive maximum.
    --- @param min number
    --- @param max number
    --- @return number
    --- @diagnostic disable-next-line: missing-return
    function rand_float(min, max) stub() end

    --- Generates a random floating-point number between an inclusive minimum and exclusive maximum, approximating a normal distribution.
    --- @param min number
    --- @param max number
    --- @return number
    --- @diagnostic disable-next-line: missing-return
    function rand_normal(min, max) stub() end

    --- Generates a string of `n` random decimal digits. If `n` is not specified, defaults to 1.
    --- @param n integer?
    --- @return string
    --- @overload fun()
    --- @overload fun(n: integer)
    --- @diagnostic disable-next-line: missing-return
    function rand_digit(n) stub() end

    --- Generates `n` unique random strings of decimal digits with lengths between `len_min` and `len_max` (both inclusive).
    --- @param n integer @ The number of codes to generate.
    --- @param len_min integer @ The inclusive minimum length of each code.
    --- @param len_max integer @ The inclusive maximum length of each code.
    --- @return string[]
    --- @diagnostic disable-next-line: missing-return
    function rand_unique_codes(n, len_min, len_max) stub() end

    --- Generates a random 32-bit signed integer.
    --- @return integer
    --- @diagnostic disable-next-line: missing-return
    function rand_int32() stub() end

    --- Returns a boolean value with probability `p` of being true, where `0.0 <= p <= 1.0`.
    --- @param p number
    --- @return boolean
    --- @diagnostic disable-next-line: missing-return
    function chance(p) stub() end

    --- Calculates a 2-dimensional Perlin noise sample corresponding to the specified coordinates and noise parameters.
    --- @param x number @ The X coordinate of the noise sample.
    --- @param y number @ The Y coordinate of the noise sample.
    --- @param octaves integer @ The number of octaves (layers) to add to the noise.
    --- @param frequency number @ The number of noise cycles per unit length.
    --- @param persistence number @ The amplitude multiplier for each successive octave.
    --- @param lacunarity number @ The frequency multiplier for each successive octave.
    --- @param seed integer @ The seed of the noise generator.
    --- @return number
    --- @diagnostic disable-next-line: missing-return
    function perlin_sample(x, y, octaves, frequency, persistence, lacunarity, seed) stub() end

    --- @param agent_id integer
    --- @param loaded boolean
    --- @diagnostic disable-next-line: missing-return
    function set_agent_sounds_loaded(agent_id, loaded) stub() end
end

function is_number(val)
    return type(val) == 'number'
end

function is_string(val)
    return type(val) == 'string'
end