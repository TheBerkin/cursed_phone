--- @class PrioritySetBucket
--- @field p number
--- @field values Set

--- A "lowest-first" priority set with unique values and fast lookups.
--- @class PrioritySet
--- @field package _buckets_sparse table<number, PrioritySetBucket>
--- @field package _buckets_sorted PrioritySetBucket[]
--- @field package _value_priorities table<any, number>
--- @field package _length integer
--- @operator len: integer
local C_PrioritySet, new_PrioritySet, M_PrioritySet = class 'PrioritySet'


--- @return PrioritySet
function PrioritySet()
    --- @type PrioritySet
    local instance = {
        _buckets_sorted = {},
        _buckets_sparse = {},
        _value_priorities = {},
        _length = 0,
    }
    return new_PrioritySet(instance)
end

--- @param a PrioritySetBucket
--- @param b PrioritySetBucket
local function cmp_buckets(a, b)
    return a.p > b.p
end

function C_PrioritySet:add(obj, priority)
    if obj == nil then return false end
    if self._value_priorities[obj] then return false end
    local bucket_index = priority or 1
    --- @type PrioritySetBucket
    local bucket = self._buckets_sparse[priority]
    if not bucket then
        bucket = { p = bucket_index, values = Set() }
        self._buckets_sparse[bucket_index] = bucket
        table.insert(self._buckets_sorted, bucket)
        table.sort(self._buckets_sorted, cmp_buckets)
    end
    if not self._value_priorities[obj] then
        self._value_priorities[obj] = bucket_index
        bucket.values:add(obj)
        self._length = self._length + 1
        return true
    end
    return false
end

--- Removes the next value from the set and returns it.
--- @return boolean @ Indicates whether a value was successfully popped.
--- @return any? @ The popped value.
--- @return number? @ The priority of the popped value.
function C_PrioritySet:pop()
    local bucket = self._buckets_sorted[1]
    if not bucket then return false, nil, nil end
    local success, value_popped = bucket.values:take()
    if #bucket.values == 0 then
        table.remove(self._buckets_sorted, 1)
        table.remove(self._buckets_sparse, bucket.p)
    end
    if success then
        self._length = self._length - 1
        table.remove(self._value_priorities, value_popped)
    end
    return success, value_popped, bucket.p
end

--- @generic T
--- @param k number
--- @param values Set
--- @param f fun(x: T, ...): (boolean?)
local function pset_foreach_impl(k, values, f, ...)
    return values:foreach(f, ...)
end

--- @generic T
--- @param f fun(x: T, ...): (boolean?)
--- @param ... any
--- @return boolean @ Indicates whether iteration was interrupted by the callback returning `true`.
function C_PrioritySet:foreach(f, ...)
    for i = 1, #self._buckets_sorted do
        local bucket = self._buckets_sorted[i]
        if bucket.values:foreach(pset_foreach_impl, f, ...) then return true end
    end
    return false
end

--- @return boolean @ Indicates whether the value was found.
--- @return number? @ The priority of the removed value, if successful.
function C_PrioritySet:remove(v)
    if v == nil then return false end
    local p = self._value_priorities[v]
    if not p then return false end
    local bucket = self._buckets_sparse[p]
    local success = bucket.values:remove(v)
    self._length = self._length - 1
    return success, p
end

function C_PrioritySet:clear()
    self._length = 0
    table.clear(self._buckets_sorted)
    table.clear(self._buckets_sparse)
end

--- @param t PrioritySet
function M_PrioritySet.__newindex(t, k, v) end

--- @param t PrioritySet
function M_PrioritySet.__len(t) return t._length end