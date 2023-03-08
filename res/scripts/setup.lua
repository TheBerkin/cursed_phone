--[[

    /==========================================================================\
    |========================= CURSED PHONE API FILE ==========================|
    |==========================================================================|
    | This script is required by phone services in order to function properly. |
    | Unless you are making changes to the engine, do not modify this file.    |
    \==========================================================================/

]]

--- Represents a unique, opaque, atomic value with no intrinsic meaning.
--- @class Symbol

--- Represents a custom ring pattern that can be assigned to an agent.
--- @class RingPattern

local M_Symbol = {
    __index = function(self, k) error("can't index a symbol", 2) end,
    __newindex = function(self, k, v) error("can't index a symbol", 2) end,
    __metatable = function(self) return nil end
}

--- Creates and returns a new symbol.
--- @return Symbol
function create_symbol()
    local s = {}
    setmetatable(s, M_Symbol)
    return s
end

function print_info()
    local is_luajit = type(jit) == 'table'
    if is_luajit then
        print("Running " .. _VERSION .. ", " .. jit.version .. "(" .. jit.arch .. ")")
    else
        print("Running " .. _VERSION .. "")
    end
end

function coerce_boolean(value)
    return not not value
end

function stub() end

print_info()