RNG_STATIC = Rng()

--- Generates a random integer between an inclusive minimum and exclusive maximum.
--- @param min integer
--- @param max integer
--- @return integer
function randi(min, max) return RNG_STATIC:int(min, max) end

--- Generates a random integer between an inclusive minimum and maximum.
--- @param min integer
--- @param max integer
--- @return integer
function irandi(min, max) return RNG_STATIC:int_i(min, max) end

--- Generates a random integer between an inclusive minimum and exclusive maximum,
--- but avoids the `skip` value within the range.
--- @param min integer
--- @param skip integer
--- @param max integer
--- @return integer
function randi_s(min, skip, max) return RNG_STATIC:int_skip(min, skip, max) end

--- Generates a random integer between an inclusive minimum and exclusive maximum
--- with an asymptotal distribution biased to lower values.
--- @param min integer
--- @param max integer
--- @return integer
function randi_l(min, max) return RNG_STATIC:int_bias_low(min, max) end

--- Generates a random integer between an inclusive minimum and exclusive maximum
--- with an asymptotal distribution biased to higher values.
--- @param min integer
--- @param max integer
--- @return integer
function randi_h(min, max) return RNG_STATIC:int_bias_high(min, max) end

--- Generates a random integer between an inclusive minimum and exclusive maximum, approximating a Gaussian (normal) distribution.
--- @param min integer
--- @param max integer
--- @return integer
function randi_g(min, max) return RNG_STATIC:int_normal(min, max) end

--- Generates a random floating-point number between an inclusive minimum and exclusive maximum.
--- @param min number
--- @param max number
--- @return number
function randf(min, max) return RNG_STATIC:float(min, max) end

--- Generates a random floating-point number between an inclusive minimum and exclusive maximum, approximating a Gaussian (normal) distribution.
--- @param min number
--- @param max number
--- @return number
function randf_g(min, max) return RNG_STATIC:normal(min, max) end

--- Generates a string of `n` random decimal digits. If `n` is not specified, defaults to 1.
--- @param n integer?
--- @return string
--- @overload fun()
--- @overload fun(n: integer)
function randd(n) return RNG_STATIC:digits(n) end

--- Generates a random 32-bit signed integer.
--- @return integer
function rand32() return RNG_STATIC:bits_32() end

--- Generates a random 64-bit signed integer.
--- @return integer
function rand64() return RNG_STATIC:bits_64() end

--- Returns a boolean value with probability `p` of being true, where `0.0 <= p <= 1.0`.
--- @param p number
--- @return boolean
function maybe(p) return RNG_STATIC:maybe(p) end