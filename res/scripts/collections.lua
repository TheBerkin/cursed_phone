--- @class Queue
--- @field private _items table
--- @field private _count integer
Queue = {}

local M_Queue = {
    __index = Queue,
    __len = function(self) return self._count end
}

--- @return Queue
function Queue.new()
    --- @type Queue
    local queue = {
        _items = {},
        _count = 0
    }
    setmetatable(queue, M_Queue)
    return queue
end

function Queue:add(item)
    local prev_length = self._count
    self._items[prev_length + 1] = item
    self._count = prev_length + 1
end

--- @return boolean
function Queue:is_empty()
    return self._count > 0
end

function Queue:clear()
    for i = 1, self._count do
        self._items[i] = nil
    end
    self._count = 0
end

--- @return integer
function Queue:count()
    return self._count
end

--- @return boolean @ Indicates whether an element was dequeued.
--- @return any? @ The element that was dequeued, if any.
function Queue:dequeue()
    local orig_length = self._count
    if orig_length == 0 then return false, nil end
    local popped = self._items[1]
    if orig_length > 1 then
        for i = 2, orig_length do
            self._items[i - 1] = self._items[i]
        end
        self._items[orig_length] = nil
        self._count = orig_length - 1
    else
        self._items[1] = nil
        self._count = 0
    end
    return true, popped
end