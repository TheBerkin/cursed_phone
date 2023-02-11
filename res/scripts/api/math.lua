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

--- Returns numbers `a` and `b` in ascending order.
--- @param a number @ The first value.
--- @param b number @ The second value.
--- @return number, number
function math.minmax(a, b)
    if a < b then
        return a, b
    else
        return b, a
    end
end

--- Linearly interpolates between `a` and `b`.
--- @param a number @ The startung value.
--- @param b number @ The ending value.
--- @param frac number @ The point between `a` and `b` to sample.
--- @param clamp boolean? @ Indicates whether to clamp `delta` between 0 and 1.  
--- @return number
function math.lerp(a, b, frac, clamp)
    if clamp then frac = math.clamp(frac, 0, 1) end 
    return (b - a) * frac + a
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

--- Remaps interpolated value `x` from its fraction between `src_a` and `src_b` to the same fraction between `dest_a` and `dest_b`.
--- @param x number @ The original interpolated value relative to `src_a` and `src_b`.
--- @param src_a number @ The starting source bound.
--- @param src_b number @ The ending source bound.
--- @param dest_a number @ The starting destination bound.
--- @param dest_b number @ The ending destination bound.
--- @param clamp boolean @ Indicates whether to clamp the interpolation fraction between 0 and 1.
function math.remap(x, src_a, src_b, dest_a, dest_b, clamp)
    return math.lerp(dest_a, dest_b, math.invlerp(x, src_a, src_b, clamp))
end