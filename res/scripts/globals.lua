--[[

    /==========================================================================\
    |========================= CURSED PHONE API FILE ==========================|
    |==========================================================================|
    | This script is required by the engine in order to function properly.     |
    | Unless you are making changes to the engine, do not modify this file.    |
    \==========================================================================/
    
]]

function is_number(val)
    return type(val) == 'number'
end

function is_string(val)
    return type(val) == 'string'
end

function is_nan(val)
    return val ~= val and type(val) == 'number'
end

--- @param val any
--- @return boolean
function coerce_boolean(val)
    return not not val
end