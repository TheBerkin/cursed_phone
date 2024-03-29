--- @meta

--- Represents a seedable pseudorandom number generator.
--- @class Rng
local C_Rng = {}

--- Creates a new RNG with an optional seed.
--- Uses a random seed if no seed is specified.
--- @param seed integer?
--- @return Rng
function Rng(seed) end

--- Generates a uniformly random integer between an inclusive minimum and exclusive maximum.
--- @param min integer
--- @param max integer
--- @return integer
function C_Rng:int(min, max) end

--- Generates a uniformly random integer between an inclusive minimum and maximum.
--- @param min integer
--- @param max integer
--- @return integer
function C_Rng:int_incl(min, max) end

--- Generates a uniformly random integer between an inclusive minimum and exclusive maximum,
--- but avoids the `skip` value within the range.
--- @param min integer
--- @param max integer
--- @param skip? integer
--- @return integer
function C_Rng:int_skip(min, max, skip) end

--- Generates a uniformly random integer between an inclusive minimum and maximum,
--- but avoids the `skip` value within the range.
--- @param min integer
--- @param max integer
--- @param skip? integer
--- @return integer
function C_Rng:int_skip_incl(min, max, skip) end

--- Generates a random integer between an inclusive minimum and exclusive maximum
--- with a distribution biased to lower values.
--- @param min integer
--- @param max integer
--- @return integer
function C_Rng:int_bias_low(min, max) end

--- Generates a random integer between an inclusive minimum and exclusive maximum
--- with a distribution biased to higher values.
--- @param min integer
--- @param max integer
--- @return integer
function C_Rng:int_bias_high(min, max) end

--- Generates a random integer between an inclusive minimum and exclusive maximum
--- with an approximated Gaussian (normal) distribution.
--- @param min integer
--- @param max integer
--- @return integer
function C_Rng:int_gaussian(min, max) end


--- Generates a random integer between an inclusive minimum and maximum, approximating a Gaussian (normal) distribution.
--- @param min integer
--- @param max integer
--- @return integer
function C_Rng:int_gaussian_incl(min, max) end

--- Generates `n` unique random integers between an inclusive minimum and maximum.
--- @param n integer @ The number of integers to generate.
--- @param min integer @ The inclusive minimum value.
--- @param max integer @ The inclusive maximum value.
--- @return integer[]
function C_Rng:ints_unique_incl(n, min, max) end

--- Generates a uniformly random floating-point number between an inclusive minimum and maximum.
--- @param min number
--- @param max number
--- @return number
function C_Rng:float(min, max) end

--- Generates a random floating-point number between an inclusive minimum and maximum, approximating a Gaussian (normal) distribution.
--- 
--- The distribution is approximated from some uniform random number *x* by the polynomial 0.5(2*x* - 1)<sup>3</sup> + 0.5.
--- @param min number
--- @param max number
--- @return number
function C_Rng:gaussian(min, max) end

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

--- Returns the input values in a randomized order.
--- @param ... any
--- @return any ...
function C_Rng:shuffle(...) end

--- Returns a random argument.
--- @param ... any
--- @return any
function C_Rng:pick(...) end