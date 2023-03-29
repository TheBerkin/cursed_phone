--- @alias PriorityQueueBucket { p: number, values: Deque }

--- A "lowest-first" priority queue.
--- @class PriorityQueue
--- @field package _buckets_sparse table<number, PriorityQueueBucket>
--- @field package _buckets_sorted PriorityQueueBucket[]
--- @field package _length integer
--- @operator len: integer
local C_PriorityQueue, new_PriorityQueue, M_PriorityQueue = class 'PriorityQueue'


--- @return PriorityQueue
function PriorityQueue()
    --- @type PriorityQueue
    local instance = {
        _buckets_sorted = {},
        _buckets_sparse = {},
        _length = 0,
    }
    return new_PriorityQueue(instance)
end

--- @param a PriorityQueueBucket
--- @param b PriorityQueueBucket
local function cmp_buckets(a, b)
    return a.p > b.p
end

function C_PriorityQueue:push_back(obj, priority)
    local bucket_index = priority or 1
    --- @type PriorityQueueBucket
    local bucket = self._buckets_sparse[priority]
    if not bucket then
        bucket = { p = bucket_index, values = Deque() }
        self._buckets_sparse[bucket] = bucket
        table.insert(self._buckets_sorted, bucket)
        table.sort(self._buckets_sorted, cmp_buckets)
    end
    self._length = self._length + 1
    bucket.values:push_back(obj)
end

--- @return boolean
--- @return any?
--- @return number?
function C_PriorityQueue:pop_front()
    local bucket = self._buckets_sorted[1]
    if not bucket then return false, nil, nil end
    local success, obj = bucket.values:pop_front()
    if #bucket.values == 0 then
        table.remove(self._buckets_sorted, 1)
        self._buckets_sparse[bucket.p] = nil
    end
    self._length = self._length - 1
    return success, obj, bucket.p
end

function C_PriorityQueue:clear()
    self._length = 0
    table.clear(self._buckets_sorted)
    table.clear(self._buckets_sparse)
end

--- @param t PriorityQueue
function M_PriorityQueue.__newindex(t, k, v) end

--- @param t PriorityQueue
function M_PriorityQueue.__len(t) return t._length end