--[[

    /==========================================================================\
    |========================= CURSED PHONE API FILE ==========================|
    |==========================================================================|
    | This script is required by phone services in order to function properly. |
    | Unless you are making changes to the engine, do not modify this file.    |
    \==========================================================================/
    
]]

--- Exposes functions to interact with and control phone services.
service = {}

local service_messages = {}

-- ========================
-- SERVICE STATE CODE CONSTANTS
-- ========================
--- @alias ServiceStateCode integer

--- @type ServiceStateCode
--- Service is idle and not in a call.
SERVICE_STATE_IDLE = 0
--- @type ServiceStateCode
--- Service is calling out.
SERVICE_STATE_CALL_OUT = 1
--- @type ServiceStateCode
--- Service is being called.
SERVICE_STATE_CALL_IN = 2
--- @type ServiceStateCode
--- Service is in a call.
SERVICE_STATE_CALL = 3

-- ========================
-- SERVICE STATUS CODE CONSTANTS
-- ========================
--- @alias ServiceIntentCode integer

--- @type ServiceIntentCode
--- Service performed no action.
SERVICE_INTENT_IDLE = 0
--- @type ServiceIntentCode
--- Service is accepting an incoming call.
SERVICE_INTENT_ACCEPT_CALL = 1
--- @type ServiceIntentCode
--- Service is hanging up.
SERVICE_INTENT_END_CALL = 2
--- @type ServiceIntentCode
--- Service is calling the user.
SERVICE_INTENT_CALL_USER = 3
--- @type ServiceIntentCode
--- Service is waiting for an operation to complete.
SERVICE_INTENT_WAIT = 4
--- @type ServiceIntentCode
--- Service is waiting for the user to dial a digit.
SERVICE_INTENT_READ_DIGIT = 5
--- @type ServiceIntentCode
--- Service is forwarding the call.
SERVICE_INTENT_FORWARD_CALL = 6
--- @type ServiceIntentCode
--- Service is finished with its current state and needs to transition to the next state.
SERVICE_INTENT_STATE_END = 7

-- ========================
-- SERVICE DATA CODE CONSTANTS
-- ========================
--- @alias ServiceDataCode integer

--- @type ServiceDataCode
--- Indicates no data was received.
local SERVICE_DATA_NONE = 0
--- Indicates that the user dialed a digit.
--- @type ServiceDataCode
local SERVICE_DATA_DIGIT = 1
--- Indicates that the user line is busy.
local SERVICE_DATA_LINE_BUSY = 2

-- ========================
-- SERVICE ROLE CONSTANTS
-- ========================
--- @alias ServiceRole integer

--- @type ServiceRole
--- A normal phone service.
SERVICE_ROLE_NORMAL = 0
--- @type ServiceRole
--- Designates a service as an intercept system.
SERVICE_ROLE_INTERCEPT = 1

--- @class PhoneServiceModule
local _PhoneServiceModule_MEMBERS = {
    tick = function(self, data_code, data)        
        local status, state = tick_service_state(self, data_code, data)
        return status, state
    end,
    transition = function(self, state)
        if state == self._state then return end
        transition_service_state(self, state)
    end,
    get_state = function(self) return self._state end,
    --- Sets the phone states during whick the idle tick will be executed.
    ---
    --- If this function is not called on a service module, idle ticks will be allowed during all phone states.
    --- @vararg PhoneStateCode
    set_idle_tick_during = function(self, ...) -- TODO: Implement set_idle_tick_during()
        local states = {...}
        -- (Is it even really worth doing any sanity checks here?)
        self._idle_tick_phone_states = states
    end,
    --- Enables or disables the ringback tone when calling the service.
    --- @param enabled boolean
    set_ringback_enabled = function(self, enabled)
        self._ringback_enabled = (not not enabled)
    end,
    --- Prints a message prefixed with the service name.
    --- @param msg any
    log = function(self, msg)
        print("[" .. self._name .. "] " .. msg)
    end,
    start = function(self)
        transition_service_state(self, SERVICE_STATE_IDLE)
    end,
    --- Sends a message to another service.
    --- @param dest_name string
    --- @param msg_type string|integer
    --- @param msg_data any
    send = function(self, dest_name, msg_type, msg_data)
        local dest_messages = service_messages[dest_name]

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
    --- @param self PhoneServiceModule
    --- @param state ServiceStateCode
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
    --- @param reason InterceptReason
    set_reason = function(self, reason)
        -- TODO: Expand InterceptReason to other kinds of reasons?
        self._reason = reason
    end,
    get_reason = function(self)
        return self._reason
    end,
    --- Check if the service has pending messages.
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

local M_PhoneServiceModule = {
    __index = function(self, index)
        return _PhoneServiceModule_MEMBERS[index]
    end
}

--- Returns an empty phone service module.
--- @param name string @The display name of the phone service
--- @param phone_number string | nil @The number associated with the phone service
--- @param role ServiceRole|nil
--- @return PhoneServiceModule
function SERVICE_MODULE(name, phone_number, role)
    assert(type(name) == 'string', "Invalid service name: expected string, but found " .. type(name))

    -- Create message queue for service
    local messages = {}
    service_messages[name] = messages

    local module = setmetatable({
        _name = name,
        _phone_number = phone_number,
        _role = role or SERVICE_ROLE_NORMAL,
        _state_coroutine = nil,
        _message_coroutine = nil,
        _state = SERVICE_STATE_IDLE,
        _state_func_tables = {},
        _idle_tick_phone_states = {},
        _ringback_enabled = true,
        _reason = INTERCEPT_NONE,
        _required_sound_banks = {},
        _is_suspended = false,
        _messages = messages
    }, M_PhoneServiceModule)

    return module
end

--- Wrapper around coroutine.yield() specifically for passing service intent to the host.
--- @param intent ServiceIntentCode
--- @param intent_data any
--- @return ServiceDataCode, any
function service.intent(intent, intent_data)
    local data_code, response_data = coroutine.yield(intent, intent_data)
    return (data_code or SERVICE_DATA_NONE), response_data
end

--- Asynchronously waits the specified number of seconds.
--- @param seconds number
function service.wait(seconds)
    local start_time = get_run_time()
    while get_run_time() - start_time < seconds do
        service.intent(SERVICE_INTENT_WAIT)
    end
end

--- Asynchronously waits the specified number of seconds or until the specified function returns true.
--- @param seconds number
--- @param predicate function
function service.wait_cancel(seconds, predicate)
    if predicate == nil or predicate() then return end
    local start_time = get_run_time()
    while not predicate() and get_run_time() - start_time < seconds do
        service.intent(SERVICE_INTENT_WAIT)
    end
end

--- Forwards the call to the specified number.
--- @param number string
function service.forward_call(number)
    service.intent(SERVICE_INTENT_FORWARD_CALL, number)
end

--- Starts a call with the user, if the line is open.
--- @return boolean
function service.start_call()
    local data_code = service.intent(SERVICE_INTENT_CALL_USER)
    return data_code ~= SERVICE_DATA_LINE_BUSY
end

--- Accepts a pending call.
function service.accept_call()
    coroutine.yield(SERVICE_INTENT_ACCEPT_CALL)
end

--- Ends the call.
function service.end_call()
    coroutine.yield(SERVICE_INTENT_END_CALL)
end

--- Asynchronously waits for the user to dial a digit, then returns the digit as a string.
--- If a timeout is specified, and no digit is entered within that time, this function returns nil.
--- @param max_seconds number|nil
--- @return string|nil
function service.read_digit(max_seconds)
    local timed = is_number(max_seconds) and max_seconds > 0
    if timed then
        local start_time = get_run_time()
        while get_run_time() - start_time < max_seconds do
            local data_code, data = service.intent(SERVICE_INTENT_READ_DIGIT)
            if data_code == SERVICE_DATA_DIGIT and type(data) == "string" then
                return data
            end
        end
        return nil
    else
        while true do
            local data_code, data = service.intent(SERVICE_INTENT_READ_DIGIT)
            if data_code == SERVICE_DATA_DIGIT and type(data) == "string" then
                return data
            end
        end
    end
end

--- Generates a service state machine coroutine.
--- @param s PhoneServiceModule
--- @param new_state ServiceStateCode
--- @param old_state PhoneStateCode
--- @return thread
local function gen_state_coroutine(s, new_state, old_state)
    local state_coroutine = coroutine.create(function()
        local old_func_table = s._state_func_tables[old_state]
        local new_func_table = s._state_func_tables[new_state]

        local on_enter = new_func_table and new_func_table.enter or empty_func
        local on_tick = new_func_table and new_func_table.tick or empty_func
        local prev_on_exit = old_func_table and old_func_table.exit or empty_func

        prev_on_exit(s)

        -- Emit SERVICE_INTENT_STATE_END
        if old_state then
            coroutine.yield(SERVICE_INTENT_STATE_END, old_state)
        end

        on_enter(s)
        while true do
            on_tick(s)
            service.intent(SERVICE_INTENT_IDLE)
        end
    end)
    return state_coroutine
end

--- @param s PhoneServiceModule
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

--- Transitions to the specified state on a service.
--- Returns true if the transition was successful; otherwise, returns false.
--- @param s PhoneServiceModule
--- @param state ServiceStateCode
--- @return boolean
function transition_service_state(s, state)
    local prev_state = s._state
    local state_coroutine = gen_state_coroutine(s, state, prev_state)
    
    s._state = state
    s._state_coroutine = state_coroutine
    return state_coroutine ~= nil
end

--- @param s PhoneServiceModule
--- @return ServiceIntentCode, any
function tick_service_state(s, data_code, data)
    -- Check if a state machine is even running
    local state_coroutine = s._state_coroutine
    local message_coroutine = s._message_coroutine

    -- Message handling takes priority over state ticks
    local active_coroutine = message_coroutine or state_coroutine

    -- If no state is active, there's no need to tick anything
    if active_coroutine == nil then
        return SERVICE_INTENT_IDLE, nil
    end

    -- If the state has finished, inform the caller that we need to transition
    if coroutine.status(state_coroutine) == 'dead' then
        return SERVICE_INTENT_STATE_END, s._state
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
        return SERVICE_INTENT_STATE_END, s._state
    end

    -- Return latest status and any associated data
    return intent or SERVICE_INTENT_IDLE, intent_data
end

--- Gets the current state of a service.
--- @param s PhoneServiceModule
--- @return ServiceStateCode
function get_service_state(s)
    return s._state or SERVICE_STATE_IDLE
end