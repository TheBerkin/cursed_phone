--- @type table<table, string>
local metatable_class_names = {}

--- @type table<string, table>
local class_metatables = {}

--- @generic T
--- @param name string @ The name to register for the class.
--- @param t T? @ The member table for the class.
--- @return T @ The member table for the class.
--- @return fun(instance: T?): T @ The instantiation function for the class.
--- @return table @ The metatable for the class.
function class(name, t)
    local c = t or {}
    local mt = {
        __index = c
    }
    metatable_class_names[mt] = name
    class_metatables[name] = mt
    local instantiate = function(obj)
        local instance = obj or {}
        setmetatable(instance, mt)
        return instance
    end
    return c, instantiate, mt
end

function xtype(object)
    return metatable_class_names[getmetatable(object)] or type(object)
end