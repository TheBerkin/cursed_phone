--[[

    /==========================================================================\
    |========================= CURSED PHONE API FILE ==========================|
    |==========================================================================|
    | This script is required by the engine in order to function properly.     |
    | Unless you are making changes to the engine, do not modify this file.    |
    \==========================================================================/
    
]]

--- Exposes functions to interact with and control the current agent.
agent = {}

local RING_PATTERN_DEFAULT = 'Q2000 L4000'

local ACTIVE_AGENT_MACHINES = {}
local M_ACTIVE_AGENT_MACHINES = { __weak = 'kv' }
setmetatable(ACTIVE_AGENT_MACHINES, M_ACTIVE_AGENT_MACHINES)

function assert_agent_caller()
    --if ACTIVE_AGENT_MACHINES[coroutine.running()] == nil then error("Function may only be called by agents", 3) end
end

local agent_messages = {}

--- @class AgentMessage
--- @field sender string
--- @field type string
--- @field data any

--- @enum AgentState
--- Defines possible state codes for agents.
AgentState = {
    --- Agent is not in a call.
    IDLE = 0,
    --- Agent is calling the host.
    CALL_OUT = 1,
    --- Agent is being called by the host.
    CALL_IN = 2,
    --- Agent is in a call.
    CALL = 3
}

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
    --- Agent wants to forward the call to a specific phone number.
    FORWARD_CALL_NUMBER = 6,
    --- Agent wants to end its current state and transition to another one.
    STATE_END = 7,
    --- Agent wants to forward the call to a specific Agent ID.
    FORWARD_CALL_AGENT_ID = 8,
    --- Agent wants the speech recognition engine to listen for and return a phrase.
    READ_PHRASE = 9,
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

--- @enum AgentRole
--- Defines available roles for agents.
AgentRole = {
    --- A normal phone agent.
    NORMAL = 0,
    --- Designates an agent as an intercept system.
    INTERCEPT = 1,
    --- Designates an agent as the Tollmaster.
    TOLLMASTER = 2
}

--- @class StateFunctionTable
--- @field enter async fun(self: AgentModule) @ Called when the state is entered.
--- @field tick async fun(self: AgentModule) @ Called each tick after `enter`.
--- @field exit async fun(self: AgentModule) @ Called when the state is exiting.
--- @field message async fun(self: AgentModule, sender: string, msg_type: string, msg_data: any) @ Called when the agent receives a message. 

--- @class AgentModule
--- @field _id integer?
--- @field _state_coroutine thread?
--- @field _message_coroutine thread?
--- @field _state AgentState
--- @field _state_func_tables table<AgentState, StateFunctionTable>
--- @field _sound_bank_states AgentState[]
--- @field _required_sound_banks table<string, boolean>
--- @field _custom_ring_pattern RingPattern?
local _AgentModule_MEMBERS = {
    tick = function(self, data_code, data)
        local status, state, continuation = tick_agent_state(self, data_code, data)
        return status, state, continuation
    end,
    transition = function(self, state)
        if state == self._state then return end
        transition_agent_state(self, state)
    end,
    get_state = function(self) return self._state end,
    --- Sets the phone states during whick the idle tick will be executed.
    ---
    --- If this function is not called on a agent module, idle ticks will be allowed during all phone states.
    --- @vararg PhoneStateCode
    set_idle_tick_during = function(self, ...) -- TODO: Implement set_idle_tick_during()
        local states = {...}
        -- (Is it even really worth doing any sanity checks here?)
        self._idle_tick_phone_states = states
    end,
    --- Enables or disables the ringback tone when calling the agent.
    --- @param enabled boolean
    set_ringback_enabled = function(self, enabled)
        self._ringback_enabled = coerce_boolean(enabled)
    end,
    --- Prints a message prefixed with the agent name.
    --- @param msg any
    log = function(self, msg)
        print("[" .. self._name .. "] " .. msg)
    end,
    --- Sets the load handler for the agent.
    --- This handler runs as soon as the agent module has finished loading.
    --- @param handler fun(self: AgentModule)
    on_load = function(self, handler)
        assert(type(handler) == 'function', "Handler must be a function")
        self._on_load = handler
    end,
    --- Sets the unload handler for the agent.
    --- This handler runs before the agent module has been unloaded on engine shutdown.
    --- @param handler fun(self: AgentModule)
    on_unload = function(self, handler)
        assert(type(handler) == 'function', "Handler must be a function")
        self._on_unload = handler
    end,
    --- Starts the agent's state machine, if it isn't already started.
    start = function(self)
        if self._state_coroutine then return false end
        transition_agent_state(self, AgentState.IDLE)
        return true
    end,
    --- Sends a message to another agent.
    --- @param dest_name string
    --- @param msg_type string|integer
    --- @param msg_data any
    send = function(self, dest_name, msg_type, msg_data)
        local dest_messages = agent_messages[dest_name]

        if dest_messages == nil then 
            print("WARN: Tried to write to nonexistent message queue: '" .. dest_name .. "'")
            return
        end

        local msg = {
            sender = self._name,
            type = msg_type,
            data = msg_data
        }
        table.insert(dest_messages, msg)
    end,
    --- Adds a function table for the specified state code.
    --- @param self AgentModule
    --- @param state AgentState
    --- @param func_table StateFunctionTable
    state = function(self, state, func_table)
        --- @diagnostic disable-next-line: undefined-field
        self._state_func_tables[state] = func_table
    end,
    --- Convenience function that calls `state` to configure a `CALL_IN` state that immediately accepts all calls.
    --- @param self AgentModule
    accept_all_calls = function(self) 
        self:state(AgentState.CALL_IN, {
            enter = function(self)
                agent.accept_call()
            end
        })
    end,
    suspend = function(self)
        self._is_suspended = true
    end,
    resume = function(self)
        self._is_suspended = false
    end,
    is_suspended = function(self)
        return self._is_suspended
    end,
    --- Clear any pending messages.
    clear_messages = function(self)
        table.clear(self._messages)
    end,
    --- Sets the call reason.
    --- @param self AgentModule
    --- @param reason CallReason
    set_call_reason = function(self, reason)
        self._call_reason = reason
    end,
    --- @return CallReason
    get_call_reason = function(self)
        return self._call_reason
    end,
    --- Sets the price to call the agent in payphone mode.
    set_custom_price = function(self, price)
        assert(is_number(price), "Price must be a number.")
        self._has_custom_price = true
        self._custom_price = price
    end,
    --- Check if the agent has pending messages.
    --- @return boolean
    has_messages = function(self)
        return #self._messages > 0
    end,
    --- Removes the oldest message from the queue and returns it.
    --- If the message queue is empty, the function returns nil.
    --- @return AgentMessage?
    pop_message = function(self)
        local messages = self._messages
        local msgc = #messages
        if msgc == 0 then return nil end
        local msg = table.remove(messages, 1)
        return msg
    end,
    --- Requires the specified sound bank during calls.
    --- @param self AgentModule
    --- @param bank_name string
    require_sound_bank = function(self, bank_name)
        self._required_sound_banks[bank_name] = true
    end,
    --- Sets the agent states during which required sound banks will be loaded.
    --- @param self AgentModule
    --- @vararg AgentState | AgentState[]?
    set_sound_banks_loaded_during = function(self, ...)
        local state_args = {...}
        local set = {}
        for _, states in pairs(state_args) do
            local t_states = type(states)
            if t_states == 'table' then
                for k, v in pairs(states) do
                    set[v] = true
                end
            elseif t_states == 'number' or t_states == 'integer' then
                set[states] = true
            end
        end
        self._sound_bank_states = set
    end,
    --- Sets the ring pattern this agent uses when they call the host.
    --- @param self AgentModule
    --- @param expr string?
    set_custom_ring_pattern = function(self, expr)
        expr = expr or RING_PATTERN_DEFAULT
        assert(type(expr) == 'string', "Ring pattern must be a string", 2)
        local success, pattern = phone.compile_ring_pattern(expr)
        if success then
            --- @cast pattern RingPattern
            self._custom_ring_pattern = pattern
        else
            self:log(string.format("Failed to parse custom ring pattern: '%s'", expr))
            self._custom_ring_pattern = nil
        end
    end
}

local M_AgentModule = {
    __index = function(self, index)
        return _AgentModule_MEMBERS[index]
    end,
    --- Gets the ID of the current agent. 
    --- Can't be called during module initialization as agents are only assigned IDs afterwards.
    id = function(self)
        assert_agent_caller()
        return self._id
    end
}

--- Returns an empty phone agent module.
--- @param name string @ The display name of the phone agent
--- @param phone_number string? @ The number associated with the phone agent
--- @param role AgentRole? @ The role of the agent in the system; defaults to regular role
--- @return AgentModule
function create_agent(name, phone_number, role)
    assert(type(name) == 'string', "Invalid agent name: expected string, but found " .. type(name))

    -- Create message queue for agent
    local messages = {}
    agent_messages[name] = messages

    --- @type AgentModule
    local module = setmetatable({
        _name = name,
        _phone_number = phone_number,
        _role = role or AgentRole.NORMAL,
        _state_coroutine = nil,
        _message_coroutine = nil,
        _prev_state = AgentState.IDLE,
        _state = AgentState.IDLE,
        _state_func_tables = {},
        _sound_bank_states = {},
        _idle_tick_phone_states = {},
        _ringback_enabled = true,
        _custom_ring_pattern = nil,
        _reason = CallReason.NONE,
        _required_sound_banks = {},
        _has_custom_price = false,
        _custom_price = 0,
        _is_suspended = false,
        _messages = messages
    }, M_AgentModule)

    module:set_sound_banks_loaded_during(AgentState.CALL_OUT, AgentState.CALL)

    return module
end

--- @async
--- Suspends execution of the current agent state until the next tick and passes an intent from the agent to the engine.
--- @param intent IntentCode
--- @param intent_data any?
--- @param should_continue boolean?
--- @return IntentResponseCode, any
function agent.intent(intent, intent_data, should_continue)
    assert_agent_caller()
    local data_code, response_data = coroutine.yield(intent, intent_data, should_continue or false)
    return (data_code or IntentResponseCode.NONE), response_data
end

--- @async
--- Asynchronously waits the specified number of seconds, or forever if no duration is specified.
--- @param seconds number?
function agent.wait(seconds)
    assert_agent_caller()
    if seconds ~= nil then
        local start_time = engine_time()
        while engine_time() - start_time < seconds do
            agent.intent(IntentCode.WAIT)
        end
    else
        while true do
            agent.intent(IntentCode.WAIT)
        end
    end
end

--- @async
--- Asynchronously waits for a dynamic number of seconds determined by calling the supplied function every tick.
--- Stops as soon as the last returned duration exceeds the current waiting time.
--- @param duration_func fun(): number @ The function that returns the amount of time to wait.
function agent.wait_dynamic(duration_func)
    assert_agent_caller()
    local start_time = engine_time()
    while true do
        local current_duration = duration_func()
        if is_number(current_duration) and engine_time() - start_time >= current_duration then return end
        agent.intent(IntentCode.WAIT)
    end
end

--- @async
--- Asynchronously waits the specified number of seconds or until the specified function returns true.
--- @param seconds number
--- @param predicate fun(): boolean
--- @return boolean @ Indicates whether the predicate returned true and canceled the waiting period.
function agent.wait_cancel(seconds, predicate)
    if predicate then
        local start_time = engine_time()
        while engine_time() - start_time < seconds do
            if predicate() then return true end
            agent.intent(IntentCode.WAIT)
        end
    else
        agent.wait(seconds)
    end
    return false
end

--- @async
--- Asynchronously waits until the specified function returns true. Function is called once per agent tick.
--- @param predicate function
function agent.wait_until(predicate)
    assert_agent_caller()
    while not predicate() do
        agent.intent(IntentCode.WAIT)
    end
end

--- @async
--- @param interval number
--- @param p number
--- @param timeout number?
function agent.chance_interval(interval, p, timeout)
    local start_time = engine_time()
    local last_interval_start_time = start_time

    if interval > 0 and chance(p) then return end

    while not timeout or engine_time() - start_time < timeout do
        local time = engine_time()
        if time - last_interval_start_time > interval then
            last_interval_start_time = last_interval_start_time + interval
            if chance(p) then return end
        end
        agent.intent(IntentCode.WAIT)
    end
end

--- @async
--- @overload fun()
--- @overload fun(n: integer)
--- Yields control from the current agent to the caller, optionally specifying a custom number of ticks to yield for.
---
--- Calling this function with `n > 1` is especially useful for rate-limiting computationally expensive tasks.
--- @param n integer? @ Indicates how many ticks to yield for. If excluded, yields once.
function agent.yield(n)
    assert_agent_caller()
    if n then
        for i = 1, n do
            coroutine.yield(IntentCode.YIELD)
        end
    else
        coroutine.yield(IntentCode.YIELD)
    end
end

--- @async
--- Runs multiple agent tasks in parallel.
--- @param ... function
function agent.multi_task(...)
    local coroutines = table.map({...}, function(f) return {
        co = coroutine.create(f),
        last_response_code = IntentResponseCode.NONE,
        last_response_data = nil
    } end)

    while true do
        local tasks_running = false
        local task_count = #coroutines

        for i = 1, task_count do
            local state = coroutines[i]
            if coroutine.status(state.co) ~= 'dead' then
                local success, intent, intent_data = coroutine.resume(state.co, state.last_response_code, state.last_response_data)
                tasks_running = true
                if success then
                    local response_code, response_data = agent.intent(intent, intent_data, i < task_count)
                    state.last_response_code = response_code
                    state.last_response_data = response_data
                end
            end
        end
        if not tasks_running then return end
    end
end

--- @async
--- Runs a task until the specified predicate (run every tick) returns a falsy value or the task ends on its own.
function agent.do_task_while(task, predicate)
    local co = coroutine.create(task)
    local last_response_code = IntentResponseCode.NONE
    local last_response_data = nil

    while predicate() do
        if coroutine.status(co) == 'dead' then return end
        local success, intent, intent_data = coroutine.resume(co, last_response_code, last_response_data)
        if success then
            last_response_code, last_response_data = agent.intent(intent, intent_data)
        end
    end
end

--- Returns the number that was used to reach the current agent.
---
--- For intercept agents, this can be any value.
--- @return string
--- @nodiscard
function agent.caller_dialed_number()
    assert_agent_caller()
    return _caller_dialed_number_impl()
end

--- @async
--- Forwards the call to the specified number.
--- @param number string
function agent.forward_call(number)
    agent.intent(IntentCode.FORWARD_CALL_NUMBER, number)
end

--- @async
--- Forwards the call to the specified agent ID.
--- @param agent_id integer
function agent.forward_call_id(agent_id)
    agent.intent(IntentCode.FORWARD_CALL_AGENT_ID, agent_id)
end

--- @async
--- Starts a call with the user, if the line is open.
--- @return boolean
function agent.start_call()
    assert_agent_caller()
    local data_code = agent.intent(IntentCode.CALL_USER)
    return data_code ~= IntentResponseCode.LINE_BUSY
end

--- @async
--- Accepts a pending call.
function agent.accept_call()
    assert_agent_caller()
    coroutine.yield(IntentCode.ACCEPT_CALL)
end

--- @async
--- Ends the call.
function agent.end_call()
    assert_agent_caller()
    coroutine.yield(IntentCode.END_CALL)
end

--- @async
--- Asynchronously waits for the user to dial a digit, then returns the digit as a string.
--- If a timeout is specified, and no digit is entered within that time, this function returns `nil`.
--- @param timeout number? @ The maximum amount of time in seconds to poll for.
--- @return string?
function agent.read_digit(timeout)
    assert_agent_caller()
    local timed = is_number(timeout) and timeout > 0
    if timed then
        local start_time = engine_time()
        while engine_time() - start_time < timeout do
            local data_code, data = agent.intent(IntentCode.READ_DIGIT)
            if data_code == IntentResponseCode.DIGIT and type(data) == "string" then
                return data
            end
        end
        return nil
    else
        while true do
            local data_code, data = agent.intent(IntentCode.READ_DIGIT)
            if data_code == IntentResponseCode.DIGIT and type(data) == "string" then
                return data
            end
        end
    end
end

--- @async
function agent.read_digits(digit_count, digit_timeout)
    assert_agent_caller()
    local digits = ""
    while #digits < digit_count do 
        local next_digit = agent.read_digit(digit_timeout)
        if not next_digit then return nil end
        digits = digits .. next_digit
    end
    return digits
end

--- Generates an agent state machine coroutine.
--- @param a AgentModule
--- @param new_state AgentState
--- @param old_state AgentState
--- @return thread
local function gen_state_coroutine(a, new_state, old_state)
    local state_coroutine = coroutine.create(function()
        local old_func_table = a._state_func_tables[old_state]
        local new_func_table = a._state_func_tables[new_state]

        local on_enter = new_func_table and new_func_table.enter or stub
        local on_tick = new_func_table and new_func_table.tick or stub
        local prev_on_exit = old_func_table and old_func_table.exit or stub

        prev_on_exit(a)

        -- Emit state-end intent
        if old_state then
            coroutine.yield(IntentCode.STATE_END, old_state)
        end

        -- Load/unload sound banks as needed
        set_agent_sounds_loaded(a._id, coerce_boolean(a._sound_bank_states[new_state]))

        on_enter(a)
        while true do
            on_tick(a)
            agent.intent(IntentCode.YIELD)
        end
    end)
    ACTIVE_AGENT_MACHINES[state_coroutine] = state_coroutine
    return state_coroutine
end

--- @param a AgentModule
--- @param msg AgentMessage
--- @return thread?
local function gen_msg_handler_coroutine(a, msg)
    local state_table = a._state_func_tables[a._state]
    local handler = state_table and state_table.message
    if not handler then return nil end

    local msg_coroutine = coroutine.create(function()
        handler(a, msg.sender, msg.type, msg.data)
        a._message_coroutine = nil
    end)

    return msg_coroutine
end

--- Transitions to the specified state on a agent.
--- Returns true if the transition was successful; otherwise, returns false.
--- @param a AgentModule
--- @param state AgentState
--- @return boolean
function transition_agent_state(a, state)
    local prev_state = a._state
    local state_coroutine = gen_state_coroutine(a, state, prev_state)
    a._state = state
    a._state_coroutine = state_coroutine
    return state_coroutine ~= nil
end

--- Ticks the state machine of the specified agent.
--- Returns 3 values:
--- 1. the next intent code
--- 2. the data associated with the intent
--- 3. a boolean indicating whether to continue ticking this agent
--- @param a AgentModule
--- @param data_code IntentResponseCode
--- @param data any
--- @return IntentCode, any, boolean
function tick_agent_state(a, data_code, data)
    -- Check if a state machine is even running
    local state_coroutine = a._state_coroutine
    local message_coroutine = a._message_coroutine

    -- Message handling takes priority over state ticks
    local active_coroutine = message_coroutine or state_coroutine

    -- If no state is active, there's no need to tick anything
    if active_coroutine == nil then
        return IntentCode.YIELD, nil, false
    end

    -- If the state has finished, inform the caller that we need to transition
    if state_coroutine and coroutine.status(state_coroutine) == 'dead' then
        return IntentCode.STATE_END, a._state, false
    end

    -- Handle messages
    if message_coroutine == nil and a:has_messages() then
        local msg = a:pop_message()
        --- @cast msg AgentMessage
        message_coroutine = gen_msg_handler_coroutine(a, msg)
        a._message_coroutine = message_coroutine
        active_coroutine = message_coroutine
    end

    -- Resume the state machine
    --- @cast active_coroutine thread
    local success, intent, intent_data, continuation = coroutine.resume(active_coroutine, data_code, data)

    -- If the coroutine is somehow dead/broken, transition the state
    if not success then
        -- TODO: Handle this in a way that doesn't cause UB
        error(intent)
        return IntentCode.STATE_END, a._state, false
    end

    -- Return latest status and any associated data
    return intent or IntentCode.YIELD, intent_data, continuation
end

--- Gets the current state of a agent.
--- @param s AgentModule
--- @return AgentState
function get_agent_state(s)
    return s._state or AgentState.IDLE
end