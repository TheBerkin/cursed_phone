--- A collection of unique values.
--- @class Set
--- @field package _values table
--- @field package _len integer
--- @operator len: integer
local C_Set, new_Set, M_Set = class 'Set'

--- @param values? any[]
--- @return Set
function Set(values)
    local value_lookup = {}
    local num_values = 0
    if values then
        for _, v in pairs(values) do
            num_values = num_values + 1
           value_lookup[v] = true
        end
    end

    --- @type Set
    local instance = {
        _values = value_lookup,
        _len = num_values
    }

    return new_Set(instance)
end

--- @return boolean
function C_Set:add(obj)
    if obj == nil or self._values[obj] == true then return false end
    self._values[obj] = true
    self._len = self._len + 1
    return true
end

--- @return boolean
function C_Set:contains(obj)
    return obj ~= nil and self._values[obj] == true
end

--- @return boolean
function C_Set:remove(obj)
    if obj == nil or self._values[obj] == nil then return false end
    self._values[obj] = nil
    self._len = self._len - 1
    return true
end

--- @generic T
--- @param f fun(x: T, ...): (boolean?)
--- @param ... any
--- @return boolean @ Indicates whether iteration was interrupted by the callback returning `true`.
function C_Set:foreach(f, ...)
    for v, _ in pairs(self._values) do
        if f(v, ...) then return true end
    end
    return false
end

--- Removes an arbitrary value from the set and returns it, preceded by a boolean indicating whether any value was found.
--- @return boolean
--- @return any?
function C_Set:take()
    if self._len <= 0 then return false, nil end
    local specimen = next(self._values)
    self:remove(specimen)
    return true, specimen
end

function C_Set:clear()
    table.clear(self._values)
    self._len = 0
end

--- @param t Set
function M_Set.__newindex(t, k, v) end

--- @param self Set
function M_Set.__len(self)
    return self._len
end