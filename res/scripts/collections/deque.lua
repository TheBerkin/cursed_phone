--- A double-ended queue that allows `nil` values.
--- @class Deque
--- @field package _head integer
--- @field package _tail integer
--- @operator len: integer
local C_Deque, new_Deque, M_Deque = class 'Deque'

--- Creates a new `Deque`.
--- @return Deque
function Deque()
    --- @type Deque
    local deque = {
        _head = 0,
        _tail = -1
    }
    return new_Deque(deque)
end

--- Pushes `object` to the back of the queue.
--- @param object any
function C_Deque:push_back(object) 
    local newtail = rawget(self, '_tail') + 1
    rawset(self, newtail, object)
    rawset(self, '_tail', newtail)
end

--- Pushes `object` to the front of the queue.
--- @param object any
function C_Deque:push_front(object) 
    local newhead = rawget(self, '_head') - 1
    rawset(self, newhead, object)
    rawset(self, '_head', newhead)
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