--- @meta

--- Represents a custom ring pattern that can be assigned to an agent.
--- @class RingPattern

--- Indicates whether the engine has developer features enabled.
--- @type boolean
DEVMODE = nil

--- Gets the number of seconds elapsed since the engine was initialized.
--- @return number
function engine_time() end

--- Gets the number of seconds elapsed since the current call started.
--- Returns 0 if no call is active.
--- @return number
function call_time() end

--- Generates a random integer between an inclusive minimum and exclusive maximum.
--- @param min integer
--- @param max integer
--- @return integer
function rand_int(min, max) end

--- Generates a random integer between an inclusive minimum and maximum.
--- @param min integer
--- @param max integer
--- @return integer
function rand_int_i(min, max) end

--- Generates a random integer between an inclusive minimum and exclusive maximum,
--- but avoids the `skip` value within the range.
--- @param min integer
--- @param skip integer
--- @param max integer
--- @return integer
function rand_int_skip(min, skip, max) end

--- Generates a random integer between an inclusive minimum and exclusive maximum
--- with an asymptotal distribution biased to lower values.
--- @param min integer
--- @param max integer
--- @return integer
function rand_int_bias_low(min, max) end

--- Generates a random integer between an inclusive minimum and exclusive maximum
--- with an asymptotal distribution biased to higher values.
--- @param min integer
--- @param max integer
--- @return integer
function rand_int_bias_high(min, max) end

--- Generates a random integer between an inclusive minimum and exclusive maximum, approximating a normal distribution.
--- @param min integer
--- @param max integer
--- @return integer
function rand_int_normal(min, max) end

--- Generates a random floating-point number between an inclusive minimum and exclusive maximum.
--- @param min number
--- @param max number
--- @return number
function rand_float(min, max) end

--- Generates a random floating-point number between an inclusive minimum and exclusive maximum, approximating a normal distribution.
--- @param min number
--- @param max number
--- @return number
function rand_normal(min, max) end

--- Generates a string of `n` random decimal digits. If `n` is not specified, defaults to 1.
--- @param n integer?
--- @return string
--- @overload fun()
--- @overload fun(n: integer)
function rand_digit(n) end

--- Generates `n` unique random strings of decimal digits with lengths between `len_min` and `len_max` (both inclusive).
--- @param n integer @ The number of codes to generate.
--- @param len_min integer @ The inclusive minimum length of each code.
--- @param len_max integer @ The inclusive maximum length of each code.
--- @return string[]
function rand_unique_codes(n, len_min, len_max) end

--- Generates a random 32-bit signed integer.
--- @return integer
function rand_int32() end

--- Returns a boolean value with probability `p` of being true, where `0.0 <= p <= 1.0`.
--- @param p number
--- @return boolean
function chance(p) end

--- Calculates a 2-dimensional Perlin noise sample corresponding to the specified coordinates and noise parameters.
--- @param x number @ The X coordinate of the noise sample.
--- @param y number @ The Y coordinate of the noise sample.
--- @param octaves integer @ The number of octaves (layers) to add to the noise.
--- @param frequency number @ The number of noise cycles per unit length.
--- @param persistence number @ The amplitude multiplier for each successive octave.
--- @param lacunarity number @ The frequency multiplier for each successive octave.
--- @param seed integer @ The seed of the noise generator.
--- @return number
function perlin_sample(x, y, octaves, frequency, persistence, lacunarity, seed) end

--- @param agent_id integer
--- @param loaded boolean
function set_agent_sounds_loaded(agent_id, loaded) end