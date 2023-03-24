--[[

    /==========================================================================\
    |========================= CURSED PHONE API FILE ==========================|
    |==========================================================================|
    | This script is required by the engine in order to function properly.     |
    | Unless you are making changes to the engine, do not modify this file.    |
    \==========================================================================/
    
]]

--- Removes all keys from table `t`.
--- @param t table
function table.clear(t)
    for k in pairs(t) do
        rawset(t, k, nil)
    end
end

--- Removes all sequential keys from table `t`.
--- @param t table
function table.iclear(t)
    for k in ipairs(t) do
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
--- @param t table
--- @param rng Rng?
function table.random_choice(t, rng)
    rng = rng or RNG_STATIC
    return t[rng:int_i(1, #t + 1)]
end

--- Shuffles the ordered elements of `t`.
--- Assumes that the indices of `t` start at 1 and are not sparse.
--- @param t table
--- @param rng Rng?
function table.shuffle(t, rng)
    local n = #t
    rng = rng or RNG_STATIC
    for i = 1, n do
        local i2 = rng:int_i(1, n)
        local v = t[i]
        local v2 = t[i2]
        t[i2] = v
        t[i] = v2
    end
end