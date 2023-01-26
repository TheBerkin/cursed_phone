--[[

    /==========================================================================\
    |========================= CURSED PHONE API FILE ==========================|
    |==========================================================================|
    | This script is required by phone agents in order to function properly.   |
    | Unless you are making changes to the engine, do not modify this file.    |
    \==========================================================================/
    
]]

--- Exposes functions to interact with and control the current agent.
agent = {}

local ACTIVE_AGENT_MACHINES = {}
local M_ACTIVE_AGENT_MACHINES = { __weak = 'kv' }
setmetatable(ACTIVE_AGENT_MACHINES, M_ACTIVE_AGENT_MACHINES)

function assert_agent_caller()
    --if ACTIVE_AGENT_MACHINES[coroutine.running()] == nil then error("Function may only be called by agents", 3) end
end

local agent_messages = {}

-- ========================
-- AGENT STATE CODE CONSTANTS
-- ========================
--- @alias AgentStateCode integer

--- @type AgentStateCode
--- Agent is idle and not in a call.
AGENT_STATE_IDLE = 0
--- @type AgentStateCode
--- Agent is calling out.
AGENT_STATE_CALL_OUT = 1
--- @type AgentStateCode
--- Agent is being called.
AGENT_STATE_CALL_IN = 2
--- @type AgentStateCode
--- Agent is in a call.
AGENT_STATE_CALL = 3

-- ========================
-- AGENT STATUS CODE CONSTANTS
-- ========================
--- @alias AgentIntentCode integer

--- @type AgentIntentCode
--- Agent performed no action.
AGENT_INTENT_IDLE = 0
--- @type AgentIntentCode
--- Agent is accepting an incoming call.
AGENT_INTENT_ACCEPT_CALL = 1
--- @type AgentIntentCode
--- Agent is hanging up.
AGENT_INTENT_END_CALL = 2
--- @type AgentIntentCode
--- Agent is calling the user.
AGENT_INTENT_CALL_USER = 3
--- @type AgentIntentCode
--- Agent is waiting for an operation to complete.
AGENT_INTENT_WAIT = 4
--- @type AgentIntentCode
--- Agent is waiting for the user to dial a digit.
AGENT_INTENT_READ_DIGIT = 5
--- @type AgentIntentCode
--- Agent is forwarding the call to another number.
AGENT_INTENT_FORWARD_CALL = 6
--- @type AgentIntentCode
--- Agent is finished with its current state and needs to transition to the next state.
AGENT_INTENT_STATE_END = 7
--- @type AgentIntentCode
--- Agent is forwarding the call to another Agent ID.
AGENT_INTENT_FORWARD_CALL_ID = 8

-- ========================
-- AGENT DATA CODE CONSTANTS
-- ========================
--- @alias AgentDataCode integer

--- @type AgentDataCode
--- Indicates no data was received.
local AGENT_DATA_NONE = 0
--- Indicates that the user dialed a digit.
--- @type AgentDataCode
local AGENT_DATA_DIGIT = 1
--- Indicates that the user line is busy.
local AGENT_DATA_LINE_BUSY = 2

-- ========================
-- AGENT ROLE CONSTANTS
-- ========================
--- @alias AgentRole integer

--- @type AgentRole
--- A normal phone agent.
AGENT_ROLE_NORMAL = 0
--- @type AgentRole
--- Designates an agent as an intercept system.
AGENT_ROLE_INTERCEPT = 1
--- @type AgentRole
--- Designates an agent as the Tollmaster.
AGENT_ROLE_TOLLMASTER = 2

--- @class AgentModule
local _AgentModule_MEMBERS = {
    tick = function(self, data_code, data)        
        local status, state = tick_agent_state(self, data_code, data)
        return status, state
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
    start = function(self)
        transition_agent_state(self, AGENT_STATE_IDLE)
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
    --- @param state AgentStateCode
    --- @param func_table table
    state = function(self, state, func_table)
        self._state_func_tables[state] = func_table
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
    --- @param reason CallReason
    set_reason = function(self, reason)
        self._reason = reason
    end,
    get_reason = function(self)
        return self._reason
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
    --- @return table|nil
    pop_message = function(self)
        local messages = self._messages
        local msgc = #messages
        if msgc == 0 then return nil end
        local msg = table.remove(messages, 1)
        return msg
    end,
    --- Requires the specified sound bank during calls.
    require_sound_bank = function(self, bank_name)
        self._required_sound_banks[bank_name] = true
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
--- @param role AgentRole? @ The role of the agent in the system
--- @return AgentModule
function AGENT_MODULE(name, phone_number, role)
    assert(type(name) == 'string', "Invalid agent name: expected string, but found " .. type(name))

    -- Create message queue for agent
    local messages = {}
    agent_messages[name] = messages

    local module = setmetatable({
        _name = name,
        _phone_number = phone_number,
        _role = role or AGENT_ROLE_NORMAL,
        _state_coroutine = nil,
        _message_coroutine = nil,
        _state = AGENT_STATE_IDLE,
        _state_func_tables = {},
        _idle_tick_phone_states = {},
        _ringback_enabled = true,
        _reason = CALL_REASON_NONE,
        _required_sound_banks = {},
        _has_custom_price = false,
        _custom_price = 0,
        _is_suspended = false,
        _messages = messages
    }, M_AgentModule)

    return module
end

--- Suspends execution of the current agent state until the next tick and passes an intent from the agent to the engine.
--- @param intent AgentIntentCode
--- @param intent_data any
--- @return AgentDataCode, any
function agent.intent(intent, intent_data)
    assert_agent_caller()
    local data_code, response_data = coroutine.yield(intent, intent_data)
    return (data_code or AGENT_DATA_NONE), response_data
end

--- Asynchronously waits the specified number of seconds, or forever if no duration is specified.
--- @param seconds number?
function agent.wait(seconds)
    assert_agent_caller()
    if seconds ~= nil then
        local start_time = engine_time()
        while engine_time() - start_time < seconds do
            agent.intent(AGENT_INTENT_WAIT)
        end
    else
        while true do
            agent.intent(AGENT_INTENT_WAIT)
        end
    end
end

--- Asynchronously waits the specified number of seconds or until the specified function returns true.
--- @param seconds number
--- @param predicate function
function agent.wait_cancel(seconds, predicate)
    if predicate == nil or predicate() then return end
    local start_time = engine_time()
    while not predicate() and engine_time() - start_time < seconds do
        agent.intent(AGENT_INTENT_WAIT)
    end
end

--- Returns the number that was used to reach the current agent.
---
--- For intercept agents, this can be any value.
function agent.caller_dialed_number()
    assert_agent_caller()
    return _caller_dialed_number_impl()
end

--- Forwards the call to the specified number.
--- @param number string
function agent.forward_call(number)
    agent.intent(AGENT_INTENT_FORWARD_CALL, number)
end

--- Forwards the call to the specified agent ID.
--- @param agent_id integer
function agent.forward_call_id(agent_id)
    agent.intent(AGENT_INTENT_FORWARD_CALL_ID, agent_id)
end

--- Starts a call with the user, if the line is open.
--- @return boolean
function agent.start_call()
    assert_agent_caller()
    local data_code = agent.intent(AGENT_INTENT_CALL_USER)
    return data_code ~= AGENT_DATA_LINE_BUSY
end

--- Accepts a pending call.
function agent.accept_call()
    assert_agent_caller()
    coroutine.yield(AGENT_INTENT_ACCEPT_CALL)
end

--- Ends the call.
function agent.end_call()
    assert_agent_caller()
    coroutine.yield(AGENT_INTENT_END_CALL)
end

--- Asynchronously waits for the user to dial a digit, then returns the digit as a string.
--- If a timeout is specified, and no digit is entered within that time, this function returns nil.
--- @param max_seconds number|nil
--- @return string|nil
function agent.read_digit(max_seconds)
    assert_agent_caller()
    local timed = is_number(max_seconds) and max_seconds > 0
    if timed then
        local start_time = engine_time()
        while engine_time() - start_time < max_seconds do
            local data_code, data = agent.intent(AGENT_INTENT_READ_DIGIT)
            if data_code == AGENT_DATA_DIGIT and type(data) == "string" then
                return data
            end
        end
        return nil
    else
        while true do
            local data_code, data = agent.intent(AGENT_INTENT_READ_DIGIT)
            if data_code == AGENT_DATA_DIGIT and type(data) == "string" then
                return data
            end
        end
    end
end

--- Generates an agent state machine coroutine.
--- @param s AgentModule
--- @param new_state AgentStateCode
--- @param old_state PhoneStateCode
--- @return thread
local function gen_state_coroutine(s, new_state, old_state)
    local state_coroutine = coroutine.create(function()
        local old_func_table = s._state_func_tables[old_state]
        local new_func_table = s._state_func_tables[new_state]

        local on_enter = new_func_table and new_func_table.enter or stub
        local on_tick = new_func_table and new_func_table.tick or stub
        local prev_on_exit = old_func_table and old_func_table.exit or stub

        prev_on_exit(s)

        -- Emit AGENT_INTENT_STATE_END
        if old_state then
            coroutine.yield(AGENT_INTENT_STATE_END, old_state)
        end

        on_enter(s)
        while true do
            on_tick(s)
            agent.intent(AGENT_INTENT_IDLE)
        end
    end)
    ACTIVE_AGENT_MACHINES[state_coroutine] = state_coroutine
    return state_coroutine
end

--- @param s AgentModule
--- @return thread
local function gen_msg_handler_coroutine(s, msg)
    local state_table = s._state_func_tables[s._state]
    local handler = state_table and state_table.message
    if not handler then return nil end

    local msg_coroutine = coroutine.create(function()
        handler(s, msg.sender, msg.type, msg.data)
        s._message_coroutine = nil
    end)

    return msg_coroutine
end

--- Transitions to the specified state on a agent.
--- Returns true if the transition was successful; otherwise, returns false.
--- @param s AgentModule
--- @param state AgentStateCode
--- @return boolean
function transition_agent_state(s, state)
    local prev_state = s._state
    local state_coroutine = gen_state_coroutine(s, state, prev_state)
    
    s._state = state
    s._state_coroutine = state_coroutine
    return state_coroutine ~= nil
end

--- @param s AgentModule
--- @return AgentIntentCode, any
function tick_agent_state(s, data_code, data)
    -- Check if a state machine is even running
    local state_coroutine = s._state_coroutine
    local message_coroutine = s._message_coroutine

    -- Message handling takes priority over state ticks
    local active_coroutine = message_coroutine or state_coroutine

    -- If no state is active, there's no need to tick anything
    if active_coroutine == nil then
        return AGENT_INTENT_IDLE, nil
    end

    -- If the state has finished, inform the caller that we need to transition
    if coroutine.status(state_coroutine) == 'dead' then
        return AGENT_INTENT_STATE_END, s._state
    end

    -- Handle messages
    if message_coroutine == nil and s:has_messages() then
        local msg = s:pop_message()
        message_coroutine = gen_msg_handler_coroutine(s, msg)
        s._message_coroutine = message_coroutine
        active_coroutine = message_coroutine
    end

    -- Resume the state machine
    local success, intent, intent_data = coroutine.resume(active_coroutine, data_code, data)

    -- If the coroutine is somehow dead/broken, transition the state
    if not success then
        -- TODO: Handle this in a way that doesn't cause UB
        error(intent)
        return AGENT_INTENT_STATE_END, s._state
    end

    -- Return latest status and any associated data
    return intent or AGENT_INTENT_IDLE, intent_data
end

--- Gets the current state of a agent.
--- @param s AgentModule
--- @return AgentStateCode
function get_agent_state(s)
    return s._state or AGENT_STATE_IDLE
end