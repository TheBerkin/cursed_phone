--[[

    /==========================================================================\
    |========================= CURSED PHONE API FILE ==========================|
    |==========================================================================|
    | This script is required by the engine in order to function properly.     |
    | Unless you are making changes to the engine, do not modify this file.    |
    \==========================================================================/
    
]]

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