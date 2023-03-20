--- @meta

--- @class Rng
local C_Rng = {}

--- Creates a new RNG with an optional seed.
--- Uses a random seed if no seed is specified.
--- @param seed integer?
--- @return Rng
function Rng(seed) end

--- Generates a random integer between an inclusive minimum and exclusive maximum.
--- @param min integer
--- @param max integer
--- @return integer
function C_Rng:int(min, max) end

--- Generates a random integer between an inclusive minimum and maximum.
--- @param min integer
--- @param max integer
--- @return integer
function C_Rng:int_i(min, max) end

--- Generates a random integer between an inclusive minimum and exclusive maximum,
--- but avoids the `skip` value within the range.
--- @param min integer
--- @param skip integer
--- @param max integer
--- @return integer
function C_Rng:int_skip(min, skip, max) end

--- Generates a random integer between an inclusive minimum and exclusive maximum
--- with an asymptotal distribution biased to lower values.
--- @param min integer
--- @param max integer
--- @return integer
function C_Rng:int_bias_low(min, max) end

--- Generates a random integer between an inclusive minimum and exclusive maximum
--- with an asymptotal distribution biased to higher values.
--- @param min integer
--- @param max integer
--- @return integer
function C_Rng:int_bias_high(min, max) end

--- Generates a random integer between an inclusive minimum and exclusive maximum, approximating a normal distribution.
--- @param min integer
--- @param max integer
--- @return integer
function C_Rng:int_normal(min, max) end

--- Generates a random floating-point number between an inclusive minimum and exclusive maximum.
--- @param min number
--- @param max number
--- @return number
function C_Rng:float(min, max) end

--- Generates a random floating-point number between an inclusive minimum and exclusive maximum, approximating a normal distribution.
--- @param min number
--- @param max number
--- @return number
function C_Rng:normal(min, max) end

--- Generates a string of `n` random decimal digits. If `n` is not specified, defaults to 1.
--- @param n integer?
--- @return string
--- @overload fun()
--- @overload fun(n: integer)
function C_Rng:digits(n) end

--- Generates a random 32-bit signed integer.
--- @return integer
function C_Rng:bits_32() end

--- Generates a random 64-bit signed integer.
--- @return integer
function C_Rng:bits_64() end

--- Returns a boolean value with probability `p` of being true, where `0.0 <= p <= 1.0`.
--- @param p number
--- @return boolean
function C_Rng:maybe(p) end