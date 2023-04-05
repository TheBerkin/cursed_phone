--- @class TaskLib
task = {}

--- @enum IntentCode
--- Defines intent types that an agent can send to the engine.
IntentCode = {
    --- Agent performed no action.
    YIELD = 0,
    --- Agent wants to accept an incoming call.
    ACCEPT_CALL = 1,
    --- Agent wants to end an ongoing call.
    END_CALL = 2,
    --- Agent wants to call the user.
    CALL_USER = 3,
    --- Agent is waiting for an operation to complete.
    WAIT = 4,
    --- Agent wants to read a digit from the user.
    READ_DIGIT = 5,
    --- Agent wants to forward the call to a specific phone number or agent handle.
    FORWARD_CALL = 6,
    --- Agent wants to end its current state and transition to another one.
    STATE_END = 7,
}

--- @enum IntentResponseCode
--- Defines response codes that an agent can receive after sending an intent.
IntentResponseCode = {
    --- Indicates no data was received.
    NONE = 0,
    --- Indicates a dialed digit.
    DIGIT = 1,
    --- Indicates that the line is currently busy.
    LINE_BUSY = 2,
    --- Indicates that a phrase was recognized.
    SPEECH = 3,
}

--- @async
--- Suspends execution of the current agent state until the next tick and passes an intent from the agent to the engine.
--- @param intent IntentCode
--- @param intent_data any?
--- @param should_continue boolean?
--- @return IntentResponseCode, any
function task.intent(intent, intent_data, should_continue)
    local data_code, response_data = coroutine.yield(intent, intent_data, should_continue or false)
    return (data_code or IntentResponseCode.NONE), response_data
end

--- @async
--- Asynchronously waits the specified number of seconds, or forever if no duration is specified.
--- @param seconds number?
function task.wait(seconds)
    if seconds ~= nil then
        local start_time = engine_time()
        while engine_time() - start_time < seconds do
            coroutine.yield(IntentCode.WAIT)
        end
    else
        while true do
            coroutine.yield(IntentCode.WAIT)
        end
    end
end

--- @async
--- Asynchronously waits until the specified function returns true. Function is called once per agent tick.
--- @param predicate fun(): boolean
function task.wait_until(predicate)
    while not predicate() do
        coroutine.yield(IntentCode.WAIT)
    end
end

--- @async
--- Asynchronously waits the specified number of seconds or until the specified function returns true.
--- @param seconds number
--- @param predicate fun(): boolean
--- @return boolean @ Indicates whether the predicate returned true and canceled the waiting period.
function task.wait_cancel(seconds, predicate)
    if predicate then
        local start_time = engine_time()
        while engine_time() - start_time < seconds do
            if predicate() then return true end
            coroutine.yield(IntentCode.WAIT)
        end
    else
        task.wait(seconds)
    end
    return false
end

--- @async
--- Asynchronously waits for a dynamic number of seconds determined by calling the supplied function every tick.
--- Stops as soon as the last returned duration exceeds the current waiting time.
--- @param duration_func fun(): number @ The function that returns the amount of time to wait.
function task.wait_dynamic(duration_func)
    assert_agent_caller()
    local start_time = engine_time()
    while true do
        local current_duration = duration_func()
        if is_number(current_duration) and engine_time() - start_time >= current_duration then return end
        coroutine.yield(IntentCode.WAIT)
    end
end

--- @async
--- @overload fun()
--- @overload fun(n: integer)
--- Yields control from the current agent to the caller, optionally specifying a custom number of ticks to yield for.
---
--- Calling this function with `n > 1` is especially useful for rate-limiting computationally expensive tasks.
--- @param n integer? @ Indicates how many ticks to yield for. If excluded, yields once.
function task.yield(n)
    if n then
        for i = 1, n do
            coroutine.yield(IntentCode.YIELD)
        end
    else
        coroutine.yield(IntentCode.YIELD)
    end
end

--- @async
--- @param interval number
--- @param p number
--- @param timeout number?
function task.chance_interval(interval, p, timeout)
    local start_time = engine_time()
    local last_interval_start_time = start_time

    if interval > 0 and maybe(p) then return end

    while not timeout or engine_time() - start_time < timeout do
        local time = engine_time()
        if time - last_interval_start_time > interval then
            last_interval_start_time = last_interval_start_time + interval
            if maybe(p) then return end
        end
        coroutine.yield(IntentCode.WAIT)
    end
end

--- @async
--- Runs multiple agent tasks in parallel.
--- @param ... function
function task.parallel(...)
    local coroutines = table.map({...}, function(f) return {
        co = coroutine.create(f),
        last_response_code = nil,
        last_response_data = nil
    } end)

    while true do
        local tasks_running, task_count = false, #coroutines

        for i = 1, task_count do
            local state = coroutines[i]
            if coroutine.status(state.co) ~= 'dead' then
                local success, intent, intent_data = coroutine.resume(state.co, state.last_response_code, state.last_response_data)
                tasks_running = true
                if success then
                    local response_code, response_data = task.intent(intent, intent_data, i < task_count)
                    state.last_response_code = response_code
                    state.last_response_data = response_data
                else
                    error(intent, 1)
                end
            end
        end
        if not tasks_running then return end
    end
end

--- @async
--- Runs multiple agent tasks in parallel, passing `obj` as an argument to each task function.
--- @param obj any @ The argument to pass to each task function.
--- @param ... async fun(obj) @ The tasks to run.
function task.parallel_param(obj, ...)
    local coroutines = table.map({...}, function(f) return {
        co = coroutine.create(f),
        started = false,
        last_response_code = nil,
        last_response_data = nil
    } end)
    
    while true do
        local tasks_running, task_count = false, #coroutines

        for i = 1, task_count do
            local state = coroutines[i]
            if coroutine.status(state.co) ~= 'dead' then
                local success, intent, intent_data
                if not state.started then
                    success, intent, intent_data = coroutine.resume(state.co, obj)
                    state.started = true
                else
                    success, intent, intent_data = coroutine.resume(state.co, state.last_response_code, state.last_response_data)
                end
                tasks_running = true
                if success then
                    local response_code, response_data = task.intent(intent, intent_data, i < task_count)
                    state.last_response_code = response_code
                    state.last_response_data = response_data
                else
                    error(intent, 1)
                end
            end
        end
        if not tasks_running then return end
    end
end

--- @async
--- Runs multiple agent tasks in parallel as long as the specified predicate returns true.
--- @param predicate fun(): boolean
--- @param ... async fun()
function task.parallel_limit(predicate, ...)
    local coroutines = table.map({...}, function(f) return {
        co = coroutine.create(f),
        last_response_code = nil,
        last_response_data = nil
    } end)

    while true do
        local tasks_running = false
        local task_count = #coroutines

        for i = 1, task_count do
            if not predicate() then return end
            local state = coroutines[i]
            if coroutine.status(state.co) ~= 'dead' then
                local success, intent, intent_data = coroutine.resume(state.co, state.last_response_code, state.last_response_data)
                tasks_running = true
                if success then
                    local response_code, response_data = task.intent(intent, intent_data, i < task_count)
                    state.last_response_code = response_code
                    state.last_response_data = response_data
                else
                    error(intent, 1)
                end
            end
        end
        if not tasks_running then return end
    end
end

--- @async
--- Runs a task until the specified predicate (run every tick) returns a falsy value or the task ends on its own.
--- @param f async fun()
--- @param predicate fun(): boolean
function task.limit(f, predicate)
    local co = coroutine.create(f)
    local last_response_code, last_response_data

    while predicate() do
        if coroutine.status(co) == 'dead' then return end
        local success, intent, intent_data = coroutine.resume(co, last_response_code, last_response_data)
        if success then
            last_response_code, last_response_data = task.intent(intent, intent_data)
        else
            error(intent, 1)
        end
    end
end

--- @async
--- Repeats a task until the specified predicate (run before each iteration) returns a falsy value.
--- If no predicate is specified, runs forever.
--- @param f async fun(delta_time: number)
--- @param predicate (fun(): boolean)?
function task.loop(f, predicate)
    local time_prev = engine_time()
    local time_current = engine_time()
    
    while not predicate or predicate() do
        local co = coroutine.create(f)
        local last_response_code = IntentResponseCode.NONE
        local last_response_data = nil
        local delta_time = time_current - time_prev
        local yielded = false
        local started = false

        while true do
            if coroutine.status(co) == 'dead' then break end
            local success, intent, intent_data
            if not started then
                success, intent, intent_data = coroutine.resume(co, delta_time)
                started = true
            else
                success, intent, intent_data = coroutine.resume(co, last_response_code, last_response_data)
            end
            if success then
                last_response_code, last_response_data = task.intent(intent, intent_data)
                yielded = true
            else
                error(intent, 1)
            end
        end

        time_prev = time_current
        time_current = engine_time()
        if not yielded then coroutine.yield(IntentCode.YIELD) end
    end
end

--- @async
--- Forwards the call to the specified number or agent handle (agent name prefixed with `@`).
--- @param destination string @ The phone number or agent handle to forward to
function task.forward_call(destination)
    task.intent(IntentCode.FORWARD_CALL, destination)
end

--- @async
--- Starts a call with the user, if the line is open.
--- @return boolean
function task.start_call()
    local data_code = task.intent(IntentCode.CALL_USER)
    return data_code ~= IntentResponseCode.LINE_BUSY
end

--- @async
--- Accepts a pending call.
function task.accept_call()
    coroutine.yield(IntentCode.ACCEPT_CALL)
end

--- @async
--- Ends the call.
function task.end_call()
    coroutine.yield(IntentCode.END_CALL)
end

--- @async
--- Asynchronously waits for the user to dial a digit, then returns the digit as a string.
--- If a timeout is specified, and no digit is entered within that time, this function returns `nil`.
--- @param timeout number? @ The maximum amount of time in seconds to poll for.
--- @return string?
function task.read_digit(timeout)
    local timed = is_number(timeout) and timeout > 0
    if timed then
        local start_time = engine_time()
        while engine_time() - start_time < timeout do
            local data_code, data = task.intent(IntentCode.READ_DIGIT)
            if data_code == IntentResponseCode.DIGIT and type(data) == "string" then
                return data
            end
        end
        return nil
    else
        while true do
            local data_code, data = task.intent(IntentCode.READ_DIGIT)
            if data_code == IntentResponseCode.DIGIT and type(data) == "string" then
                return data
            end
        end
    end
end

--- @async
--- @return string?
function task.read_digits(digit_count, digit_timeout)
    local digits = ""
    while #digits < digit_count do 
        local next_digit = task.read_digit(digit_timeout)
        if not next_digit then return nil end
        digits = digits .. next_digit
    end
    return digits
end

--- @async
--- Asynchronously runs the specified job until the schedule runs out of jobs.
--- @param cron_expr string @ The cron expression for the schedule.
--- @param job async fun() @ The job to run.
function task.job(cron_expr, job)
    assert(type(cron_expr) == 'string', "cron schedule expression must be a string")
    local schedule = CronSchedule(cron_expr)
    if not schedule then error("invalid cron schedule expression: '" .. cron_expr .. "'") end
    local has_jobs = true
    local job_triggered = false
    repeat
        has_jobs, job_triggered = schedule:tick()
        if job_triggered then
            job()
        end
        task.intent(IntentCode.YIELD)
    until not has_jobs
end