GLOBAL_RNG = Rng()

--- Generates a uniformly random integer between an inclusive minimum and exclusive maximum.
---
--- Shortcut for `GLOBAL_RNG:int(min, max)`.
--- @param min integer
--- @param max integer
--- @return integer
function randi(min, max) return GLOBAL_RNG:int(min, max) end

--- Generates a uniformly random integer between an inclusive minimum and maximum.
---
--- Shortcut for `GLOBAL_RNG:int_i(min, max)`.
--- @param min integer
--- @param max integer
--- @return integer
function randi_incl(min, max) return GLOBAL_RNG:int_incl(min, max) end

--- Generates a uniformly random integer between an inclusive minimum and exclusive maximum,
--- but avoids the `skip` value within the range.
---
--- Shortcut for `GLOBAL_RNG:int_skip(min, max, skip)`.
--- @param min integer
--- @param max integer
--- @param skip integer
--- @return integer
function randi_s(min, max, skip) return GLOBAL_RNG:int_skip(min, max, skip) end

--- Generates a uniformly random integer between an inclusive minimum and maximum,
--- but avoids the `skip` value within the range.
---
--- Shortcut for `GLOBAL_RNG:int_skip_incl(min, max, skip)`.
--- @param min integer
--- @param max integer
--- @param skip integer
--- @return integer
function randi_s_incl(min, max, skip) return GLOBAL_RNG:int_skip_incl(min, max, skip) end

--- Generates a random integer between an inclusive minimum and exclusive maximum
--- with a distribution biased to lower values.
---
--- Shortcut for `GLOBAL_RNG:int_bias_low(min, max)`.
--- @param min integer
--- @param max integer
--- @return integer
function randi_l(min, max) return GLOBAL_RNG:int_bias_low(min, max) end

--- Generates a random integer between an inclusive minimum and exclusive maximum
--- with a distribution biased to higher values.
---
--- Shortcut for `GLOBAL_RNG:int_bias_high(min, max)`.
--- @param min integer
--- @param max integer
--- @return integer
function randi_h(min, max) return GLOBAL_RNG:int_bias_high(min, max) end

--- Generates a random integer between an inclusive minimum and exclusive maximum, approximating a Gaussian (normal) distribution.
---
--- Shortcut for `GLOBAL_RNG:int_gaussian(min, max)`.
--- @param min integer
--- @param max integer
--- @return integer
function randi_g(min, max) return GLOBAL_RNG:int_gaussian(min, max) end

--- Generates a random integer between an inclusive minimum and maximum, approximating a Gaussian (normal) distribution.
---
--- Shortcut for `GLOBAL_RNG:int_gaussian_incl(min, max)`.
--- @param min integer
--- @param max integer
--- @return integer
function randi_g_incl(min, max) return GLOBAL_RNG:int_gaussian_incl(min, max) end

--- Generates a random floating-point number between an inclusive minimum and exclusive maximum.
---
--- Shortcut for `GLOBAL_RNG:float(min, max)`.
--- @param min number
--- @param max number
--- @return number
function randf(min, max) return GLOBAL_RNG:float(min, max) end

--- Generates a random floating-point number between an inclusive minimum and exclusive maximum, approximating a Gaussian (normal) distribution.
---
--- Shortcut for `GLOBAL_RNG:gaussian(min, max)`.
--- @param min number
--- @param max number
--- @return number
function randf_g(min, max) return GLOBAL_RNG:gaussian(min, max) end

--- Generates a string of `n` random decimal digits. If `n` is not specified, defaults to 1.
---
--- Shortcut for `GLOBAL_RNG:digits(n)`.
--- @param n integer?
--- @return string
--- @overload fun()
--- @overload fun(n: integer)
function randd(n) return GLOBAL_RNG:digits(n) end

--- Generates a random 32-bit signed integer.
---
--- Shortcut for `GLOBAL_RNG:bits_32()`.
--- @return integer
function rand32() return GLOBAL_RNG:bits_32() end

--- Generates a random 64-bit signed integer.
---
--- Shortcut for `GLOBAL_RNG:bits_64()`.
--- @return integer
function rand64() return GLOBAL_RNG:bits_64() end

--- Returns a boolean value with probability `p` of being true, where `0.0 <= p <= 1.0`.
---
--- Shortcut for `GLOBAL_RNG:maybe(p)`.
--- @param p number
--- @return boolean
function maybe(p) return GLOBAL_RNG:maybe(p) end