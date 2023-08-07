--[[

    /==========================================================================\
    |========================= CURSED PHONE API FILE ==========================|
    |==========================================================================|
    | This script is required by the engine in order to function properly.     |
    | Unless you are making changes to the engine, do not modify this file.    |
    \==========================================================================/
    
]]

--- Alias for `coroutine.yield`
yield = coroutine.yield

function is_number(val)
    return type(val) == 'number'
end

function is_string(val)
    return type(val) == 'string'
end

function is_nan(val)
    return val ~= val and type(val) == 'number'
end

function is_some(val)
    return val ~= nil
end

--- @param val any
--- @return boolean
function coerce_boolean(val)
    return not not val
end

--- Converts decibels (dB) to an amplitude scale factor.
function scale_db(db)
    return math.pow(10.0, db / 20.0)
end

--- Returns a parameterless function that calls `task_func` with the specified arguments.
--- @param fn function
--- @return fun()
function curry(fn, ...)
    local args = table.pack(...)
    return function()
        fn(table.unpack(args))
    end
end

--- @generic K, V
--- @param t table<K, V>
--- @param f fun(k: K, v: V, ...): (boolean?)
--- @return boolean @ Indicates whether iteration was interrupted by the callback returning `true`.
function foreach(t, f, ...)
    for k, v in pairs(t) do
        if f(k, v, ...) then return true end
    end
    return false
end

local _publish

if not publish_engine_event or publish_engine_event ~= _publish then
    local publish_prev = publish_engine_event
    _publish = function(event_name, ...)
        if publish_prev ~= _publish then
            publish_prev(event_name, ...)
        end
        hook.publish(event_name, ...)
    end
    publish_engine_event = _publish
end