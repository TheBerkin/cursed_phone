--- A double-ended queue that allows `nil` values.
--- @class Deque
--- @field package _head integer
--- @field package _tail integer
--- @operator len: integer
local C_Deque, new_Deque, M_Deque = class 'Deque'

local NORMALIZE_INDEX_THRESHOLD = 100

--- Creates a new `Deque`.
--- @return Deque
function Deque()
    --- @type Deque
    local deque = {
        _head = 1,
        _tail = 0
    }
    return new_Deque(deque)
end

--- @param deque Deque
local function normalize_indices(deque)
    local n = deque._tail - deque._head
    if deque._head >= NORMALIZE_INDEX_THRESHOLD then
        for i = 1, n do
            deque[i] = deque[deque._head + i - 1]
        end
        deque._head = 1
        deque._tail = n
    elseif deque._tail <= -NORMALIZE_INDEX_THRESHOLD then
        for i = 0, n - 1 do
            deque[i] = deque[deque._tail - i]
        end
        deque._head = 1
        deque._tail = n
    end
end

--- @return any[]
function C_Deque:to_table()
    local t = {}
    for i = 0, #self - 1 do
        table.insert(t, self[self._head + i])
    end
    return t
end


--- Pushes `object` to the back of the queue.
--- @param object any
function C_Deque:push_back(object)
    local newtail = rawget(self, '_tail') + 1
    rawset(self, newtail, object)
    rawset(self, '_tail', newtail)
    normalize_indices(self)
end

--- Pushes `object` to the front of the queue.
--- @param object any
function C_Deque:push_front(object) 
    local newhead = rawget(self, '_head') - 1
    rawset(self, newhead, object)
    rawset(self, '_head', newhead)
    normalize_indices(self)
end

--- Removes the frontmost element from the queue and returns it. 
---
--- Returns `true, element` if something was popped; otherwise, returns `false`. 
--- @return boolean @ Indicates whether anything was popped.
--- @return any @ The value that was popped (only populated when first return value is `true`.)
function C_Deque:pop_front()
    if self._head > self._tail then return false, nil end
    local oldhead = rawget(self, '_head')
    local value = rawget(self, oldhead)
    rawset(self, '_head', oldhead + 1)
    normalize_indices(self)
    return true, value
end

--- Removes the backmost element from the queue and returns it. 
---
--- Returns `true, element` if something was popped; otherwise, returns `false`. 
--- @return boolean @ Indicates whether anything was popped.
--- @return any @ The value that was popped (only populated when first return value is `true`.)
function C_Deque:pop_back()
    if self._head > self._tail then return false, nil end
    local oldtail = rawget(self, '_tail')
    local value = rawget(self, oldtail)
    rawset(self, '_tail', oldtail - 1)
    normalize_indices(self)
    return true, value
end

--- Removes all elements from the queue.
function C_Deque:clear()
    for i = self._head, self._tail do
        rawset(self, i, nil)
    end
    rawset(self, '_head', 0)
    rawset(self, '_tail', -1)
end

function M_Deque.__newindex(deque) end

function M_Deque.__len(deque)
    return rawget(deque, '_tail') - rawget(deque, '_head') + 1
end