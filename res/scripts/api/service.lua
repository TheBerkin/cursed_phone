--[[

    /==========================================================================\
    |========================= CURSED PHONE API FILE ==========================|
    |==========================================================================|
    | This script is required by phone services in order to function properly. |
    | Unless you are making changes to the engine, do not modify this file.    |
    \==========================================================================/
    
]]

service = {}

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
--- @alias ServiceStatusCode integer

--- @type ServiceStatusCode
--- Service performed no action.
SERVICE_STATUS_IDLE = 0
--- @type ServiceStatusCode
--- Service is accepting an incoming call.
SERVICE_STATUS_ACCEPT_CALL = 1
--- @type ServiceStatusCode
--- Service is hanging up.
SERVICE_STATUS_END_CALL = 2
--- @type ServiceStatusCode
--- Service is calling the user.
SERVICE_STATUS_CALL_USER = 3
--- @type ServiceStatusCode
--- Service is waiting for an operation to complete.
SERVICE_STATUS_WAITING = 4
--- @type ServiceStatusCode
--- Service is waiting for the user to dial a digit.
SERVICE_STATUS_REQUEST_DIGIT = 5
--- @type ServiceStatusCode
--- Service is forwarding the call.
SERVICE_STATUS_FORWARD = 6
--- @type ServiceStatusCode
--- Service is finished with its current state and needs to transition to the next state.
SERVICE_STATUS_FINISHED_STATE = 7

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
local SERVICE_DATA_USER_BUSY = 2

-- ========================
-- SERVICE ROLE CONSTANTS
-- ========================
--- @alias ServiceRole integer

--- @type ServiceRole
--- A normal phone service.
SERVICE_ROLE_NORMAL = 0
--- @type ServiceRole
--- An intercept service.
SERVICE_ROLE_INTERCEPT = 1

--- @class PhoneServiceModule
local _PhoneServiceModule_MEMBERS = {
    tick = function(self)        
        local status, state = tick_service_state(self)
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
    set_idle_tick_during = function(self, ...)
        local states = {...}
        -- (Is it even really worth doing any sanity checks here?)
        self._idle_tick_phone_states = states
    end,
    start = function(self)
        transition_service_state(self, SERVICE_STATE_IDLE)
    end,
    --- Adds a function table for the specified state code.
    --- @param self PhoneServiceModule
    --- @param state ServiceStateCode
    --- @param func_table table
    state = function(self, state, func_table)
        self._state_func_tables[state] = func_table
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
    local module = setmetatable({
        _name = name,
        _phone_number = phone_number,
        _role = role or SERVICE_ROLE_NORMAL,
        _state_coroutine = nil,
        _state = SERVICE_STATE_IDLE,
        _state_func_tables = {},
        _idle_tick_phone_states = {}
    }, M_PhoneServiceModule)
    return module
end

--- Wrapper around coroutine.yield() specifically for passing service status to the host.
--- @param status ServiceStatusCode
--- @param status_data any
--- @return ServiceDataCode, any
function service.status(status, status_data)
    local data_code, response_data = coroutine.yield(status, status_data)
    return data_code or SERVICE_DATA_NONE, response_data
end

--- Asynchronously waits the specified number of seconds.
--- @param seconds number
function service.wait(seconds)
    local start_time = get_run_time()
    while get_run_time() - start_time < seconds do
        service.status(SERVICE_STATUS_WAITING)
    end
end

--- Forwards the call to the specified number.
--- @param number string
function service.forward_call(number)
    service.status(SERVICE_STATUS_FORWARD, number)
end

--- Starts a call with the user.
--- @return boolean
function service.start_call()
    local data_code = service.status(SERVICE_STATUS_CALL_USER)
    return data_code ~= SERVICE_DATA_USER_BUSY
end

--- Accepts a pending call.
function service.accept_call()
    service.status(SERVICE_STATUS_ACCEPT_CALL)
end

--- Ends the call.
function service.end_call()
    coroutine.yield(SERVICE_STATUS_END_CALL)
end

--- Asynchronously waits for the user to dial a digit, then returns the digit as a string.
--- If a timeout is specified, and no digit is entered within that time, this function returns nil.
--- @param max_seconds number|nil
--- @return string|nil
function service.get_digit(max_seconds)
    local timed = type(max_seconds) == "number" and max_seconds > 0
    if timed then
        while true do
            local data_code, data = service.status(SERVICE_STATUS_REQUEST_DIGIT)
            if data_code == SERVICE_DATA_DIGIT and type(data) == "string" then
                return data
            end
        end
    else
        local start_time = get_run_time()
        while get_run_time() - start_time < max_seconds do
            local data_code, data = service.status(SERVICE_STATUS_REQUEST_DIGIT)
            if data_code == SERVICE_DATA_DIGIT and type(data) == "string" then
                return data
            end
        end
        return nil
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

        local on_enter = new_func_table.enter or empty_func
        local on_tick = new_func_table.tick or empty_func
        local prev_on_exit = old_func_table and old_func_table.exit or empty_func

        prev_on_exit(s)
        on_enter(s)
        while true do
            on_tick(s)
            service.status(SERVICE_STATUS_IDLE)
        end
    end)
    return state_coroutine
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
--- @return ServiceStatusCode, any
function tick_service_state(s)
    local state_coroutine = s._state_coroutine
    if state_coroutine == nil then     
        return SERVICE_STATUS_IDLE, nil 
    end
    
    if coroutine.status(state_coroutine) ~= "dead" then
        local success, status, status_data = coroutine.resume(state_coroutine)
        -- If the coroutine is somehow dead/broken, transition the state
        if not success then
            error(status)
            return SERVICE_STATUS_FINISHED_STATE, nil
        end
        
        -- Check if the state finished, and if so, transition it
        if status == SERVICE_STATUS_FINISHED_STATE and status_data then
            local new_service_state = status_data
            transition_service_state(s, new_service_state)
        end

        -- Return latest status and any associated data
        return status or SERVICE_STATUS_IDLE, status_data
    else
        return SERVICE_STATUS_FINISHED_STATE, nil
    end
end

--- Gets the current state of a service.
--- @param s PhoneServiceModule
--- @return ServiceStateCode
function get_service_state(s)
    return s._state or SERVICE_STATE_IDLE
end