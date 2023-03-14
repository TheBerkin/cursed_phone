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
function table.random_choice(t)
    return t[rand_int(1, #t + 1)]
end