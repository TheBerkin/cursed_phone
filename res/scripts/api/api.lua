--[[

    /==========================================================================\
    |========================= CURSED PHONE API FILE ==========================|
    |==========================================================================|
    | This script is required by phone services in order to function properly. |
    | Unless you are making changes to the engine, do not modify this file.    |
    \==========================================================================/
    
]]

--------------------
---- Native API ----
--------------------

NATIVE_API(function()
    --- Gets the number of seconds elapsed since the engine was initialized.
    --- @return number
    function get_run_time() end

    --- Pauses execution for the specified number of milliseconds.
    --- @param ms integer
    --- @type function
    function sleep(ms) end

    --- Generates a random number between an inclusive minimum and exclusive maximum.
    --- @param min integer
    --- @param max integer
    --- @type function
    function random_int(min, max) end

    --- Generates a random floating-point number between an inclusive minimum and exclusive maximum.
    --- @param min number
    --- @param max number
    --- @type function
    function random_float(min, max) end
end)

--- Removes all keys from table `t`.
--- @param t table
function table.clear(t)
    for k in pairs(t) do
        rawset(t, k, nil)
    end
end