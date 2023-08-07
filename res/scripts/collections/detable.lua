--- @generic A, B
--- @class Detable<A, B>
--- @field _len integer
--- @field _table_ab table<A, B>
--- @field _table_ba table<B, A>
C_Detable, new_Detable, M_Detable = class 'Detable'

--- @return Detable
function Detable()
    --- @type Detable
    local instance = {
        _len = 0,
        _table_ab = {},
        _table_ba = {}
    }

    return new_Detable(instance)
end

--- @param a A
--- @return B?
function C_Detable:get_ab(a)
    if a == nil then return nil end
    return self._table_ab[a]
end

--- @param b B
--- @return A?
function C_Detable:get_ba(b)
    if b == nil then return nil end
    return self._table_ba[b]
end

--- @param a A
--- @param b B
function C_Detable:set_ab(a, b)
    local t_ab = self._table_ab
    local t_ba = self._table_ba
    local a_new, b_new = a, b
    local a_old, b_old = t_ba[b], t_ab[a]
    t_ab[a] = b
    t_ba[b] = a
end

local function detable_foreach_impl(k, v, f, ...)

end


--- @generic A, B
--- @param callback fun(a: A, b: B, ...): (boolean?)
--- @param ... any
--- @return boolean @ Indicates whether iteration was interrupted by the callback returning `true`.
function C_Detable:foreach(callback, ...)
    assert_type(callback, 'function', "callback must be a function")
    return foreach(self._table_ab, callback)
end

--- @param self Detable
function M_Detable.__len(self)
    return self._len
end

function M_Detable.__newindex(self, k, v) end