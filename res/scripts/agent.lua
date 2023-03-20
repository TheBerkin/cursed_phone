--[[

    /==========================================================================\
    |========================= CURSED PHONE API FILE ==========================|
    |==========================================================================|
    | This script is required by the engine in order to function properly.     |
    | Unless you are making changes to the engine, do not modify this file.    |
    \==========================================================================/
    
]]

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

--- @alias AgentMessageKey string|integer
--- @alias AgentMessageHandlerFunction async fun(self: AgentModule, sender: string, msg_type: string, msg_data: any)

--- @class StateFunctionTable
--- @field enter async fun(self: AgentModule) @ Called when the state is entered.
--- @field tick async fun(self: AgentModule) @ Called each tick after `enter`.
--- @field exit async fun(self: AgentModule) @ Called when the state is exiting.
--- @field message AgentMessageHandlerFunction | table<AgentMessageKey, AgentMessageHandlerFunction>  @ Called when the agent receives a message. 

--- @class AgentModule
--- @field package _name string
--- @field package _id integer?
--- @field package _state_coroutine thread?
--- @field package _messages AgentMessage[]
--- @field package _message_coroutine thread?
--- @field package _state AgentState
--- @field package _state_func_tables table<AgentState, StateFunctionTable>
--- @field package _sound_bank_states AgentState[]
--- @field package _required_sound_banks table<string, boolean>
--- @field package _custom_ring_pattern RingPattern?
local C_AgentModule = {}

local M_AgentModule = {
    __index = C_AgentModule
}

--- Creates a new phone agent module.
--- @param name string @ The display name of the phone agent
--- @param phone_number string? @ The number associated with the phone agent
--- @param role AgentRole? @ The role of the agent in the system; defaults to regular role
--- @return AgentModule
function AgentModule(name, phone_number, role)
    assert(type(name) == 'string', "Invalid agent name: expected string, but found " .. type(name))

    -- Create message queue for agent
    local messages = {}
    agent_messages[name] = messages

    --- @type AgentModule
    local agent = setmetatable({
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

    agent:set_sound_banks_loaded_during(AgentState.CALL_OUT, AgentState.CALL)

    return agent
end

function C_AgentModule:tick(data_code, data)
    local status, state, continuation = tick_agent_state(self, data_code, data)
    return status, state, continuation
end

function C_AgentModule:transition(state)
    if state == self._state then return end
    transition_agent_state(self, state)
end

function C_AgentModule:get_state() return self._state end

--- Enables or disables the ringback tone when calling the agent.
--- @param enabled boolean
function C_AgentModule:set_ringback_enabled(enabled)
    self._ringback_enabled = enabled
end

--- Sets the load handler for the agent.
--- This handler runs as soon as the agent module has finished loading.
--- @param handler fun(self: AgentModule)
function C_AgentModule:on_load(handler)
    assert(type(handler) == 'function', "Handler must be a function")
    self._on_load = handler
end

--- Sets the unload handler for the agent.
--- This handler runs before the agent module has been unloaded on engine shutdown.
--- @param handler fun(self: AgentModule)
function C_AgentModule:on_unload(handler)
    assert(type(handler) == 'function', "Handler must be a function")
    self._on_unload = handler
end

--- Starts the agent's state machine, if it isn't already started.
function C_AgentModule:start()
    if self._state_coroutine then return false end
    transition_agent_state(self, AgentState.IDLE)
    return true
end

--- Sends a message to another agent.
--- @param dest_name string
--- @param msg_type AgentMessageKey
--- @param msg_data any?
function C_AgentModule:send(dest_name, msg_type, msg_data)
    assert(msg_type ~= nil, "Message type cannot be nil")

    local dest_messages = agent_messages[dest_name]

    if not dest_messages then
        log.warn_caller(1, "Tried to write to nonexistent message queue: '" .. dest_name .. "'")
        return
    end

    local msg = {
        sender = self._name,
        type = msg_type,
        data = msg_data
    }
    table.insert(dest_messages, msg)
end

--- Adds a function table for the specified state code.
--- @param state AgentState
--- @param func_table StateFunctionTable
function C_AgentModule:state(state, func_table)
    --- @diagnostic disable-next-line: undefined-field
    self._state_func_tables[state] = func_table
end

--- Convenience function that calls `state` to configure a `CALL_IN` state that immediately accepts all calls.
function C_AgentModule:accept_all_calls() 
    self:state(AgentState.CALL_IN, {
        enter = function(self)
            task.accept_call()
        end
    })
end

function C_AgentModule:suspend()
    self._is_suspended = true
end

function C_AgentModule:resume()
    self._is_suspended = false
end

function C_AgentModule:is_suspended()
    return self._is_suspended
end

--- Clear any pending messages.
function C_AgentModule:clear_messages()
    table.clear(self._messages)
end

--- Sets the call reason.
--- @param reason CallReason
function C_AgentModule:set_call_reason(reason)
    self._call_reason = reason
end

--- @return CallReason
function C_AgentModule:get_call_reason()
    return self._call_reason
end

--- Sets the price to call the agent in payphone mode.
--- @param price number
function C_AgentModule:set_custom_price(price)
    assert(is_number(price), "Price must be a number.")
    self._has_custom_price = true
    self._custom_price = price
end

--- Check if the agent has pending messages.
--- @return boolean
function C_AgentModule:has_messages()
    return #self._messages > 0
end

--- Removes the oldest message from the queue and returns it.
--- If the message queue is empty, the function returns nil.
--- @return AgentMessage?
function C_AgentModule:pop_message()
    local messages = self._messages
    local msgc = #messages
    if msgc == 0 then return nil end
    local msg = table.remove(messages, 1)
    return msg
end

--- Requires the specified sound bank during calls.
--- @param bank_name string
function C_AgentModule:require_sound_bank(bank_name)
    self._required_sound_banks[bank_name] = true
end

--- Sets the agent states during which required sound banks will be loaded.
--- @vararg AgentState | AgentState[]?
function C_AgentModule:set_sound_banks_loaded_during(...)
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
end

--- Sets the ring pattern this agent uses when they call the host.
--- @param expr string?
function C_AgentModule:set_custom_ring_pattern(expr)
    expr = expr or RING_PATTERN_DEFAULT
    assert(type(expr) == 'string', "Ring pattern must be a string", 2)
    local success, pattern = phone.compile_ring_pattern(expr)
    if success then
        --- @cast pattern RingPattern
        self._custom_ring_pattern = pattern
    else
        log.warn(string.format("Failed to parse custom ring pattern: '%s'", expr))
        self._custom_ring_pattern = nil
    end
end

function C_AgentModule:id()
    --- Gets the ID of the current agent. 
    --- Can't be called during module initialization as agents are only assigned IDs afterwards.
    return self._id
end

local function handler_stub() end

--- @package
--- Generates an agent state machine coroutine.
--- @param new_state AgentState
--- @param old_state AgentState
--- @return thread
function C_AgentModule:gen_state_coroutine(new_state, old_state)
    local state_coroutine = coroutine.create(function()
        local old_func_table = self._state_func_tables[old_state]
        local new_func_table = self._state_func_tables[new_state]
    
        local on_enter = new_func_table and new_func_table.enter or handler_stub
        local on_tick = new_func_table and new_func_table.tick or handler_stub
        local prev_on_exit = old_func_table and old_func_table.exit or handler_stub
    
        prev_on_exit(self)
    
        -- Emit state-end intent
        if old_state then
            coroutine.yield(IntentCode.STATE_END, old_state)
        end
    
        -- Load/unload sound banks as needed
        set_agent_sounds_loaded(self._id, self._sound_bank_states[new_state] ~= nil)
    
        on_enter(self)
        while true do
            on_tick(self)
            task.intent(IntentCode.YIELD)
        end
    end)
    ACTIVE_AGENT_MACHINES[state_coroutine] = state_coroutine
    return state_coroutine
end

--- @package
--- @param msg AgentMessage
--- @return thread?
function C_AgentModule:gen_msg_handler_coroutine(msg)
    local state_table = self._state_func_tables[self._state]
    local handler = state_table and state_table.message
    local handler_type = type(handler)
    local handler_func

    if handler_type == 'function' then
        handler_func = handler
    elseif handler_type == 'table' then
        handler_func = handler[msg.type]
    else
        return nil
    end

    if type(handler_func) ~= 'function' then return nil end

    local msg_coroutine = coroutine.create(function()
        handler_func(self, msg.sender, msg.type, msg.data)
        self._message_coroutine = nil
    end)

    return msg_coroutine
end

--- Transitions to the specified state on a agent.
--- Returns true if the transition was successful; otherwise, returns false.
--- @param agent AgentModule
--- @param state AgentState
--- @return boolean
function transition_agent_state(agent, state)
    local prev_state = agent._state
    local state_coroutine = agent:gen_state_coroutine(state, prev_state)
    agent._state = state
    agent._state_coroutine = state_coroutine
    return state_coroutine ~= nil
end

--- Ticks the state machine of the specified agent.
--- Returns 3 values:
--- 1. the next intent code
--- 2. the data associated with the intent
--- 3. a boolean indicating whether to continue ticking this agent
--- @param agent AgentModule
--- @param data_code IntentResponseCode
--- @param data any
--- @return IntentCode, any, boolean
function tick_agent_state(agent, data_code, data)
    -- Check if a state machine is even running
    local state_coroutine = agent._state_coroutine
    local message_coroutine = agent._message_coroutine

    -- Message handling takes priority over state ticks
    local active_coroutine = message_coroutine or state_coroutine

    -- If no state is active, there's no need to tick anything
    if active_coroutine == nil then
        return IntentCode.YIELD, nil, false
    end

    -- If the state has finished, inform the caller that we need to transition
    if state_coroutine and coroutine.status(state_coroutine) == 'dead' then
        return IntentCode.STATE_END, agent._state, false
    end

    -- Handle messages
    if message_coroutine == nil and agent:has_messages() then
        local msg = agent:pop_message()
        --- @cast msg AgentMessage
        message_coroutine = agent:gen_msg_handler_coroutine(msg)
        if message_coroutine then
            agent._message_coroutine = message_coroutine
            active_coroutine = message_coroutine
        end
    end

    -- Resume the state machine
    --- @cast active_coroutine thread
    local success, intent, intent_data, continuation = coroutine.resume(active_coroutine, data_code, data)

    -- If the coroutine is somehow dead/broken, transition the state
    if not success then
        -- TODO: Handle this in a way that doesn't cause UB
        error(intent)
        return IntentCode.STATE_END, agent._state, false
    end

    -- Return latest status and any associated data
    return intent or IntentCode.YIELD, intent_data, continuation
end

--- Gets the current state of a agent.
--- @param agent AgentModule
--- @return AgentState
function get_agent_state(agent)
    return agent._state or AgentState.IDLE
end