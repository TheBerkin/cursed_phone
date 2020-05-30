--[[
=================================
===== CURSED PHONE API FILE =====
=================================

This script is required by all phone services in order to function properly.
Unless you are making changes to the engine, do not modify this file.

]]

-- ========================
-- SOUND CHANNEL CONSTANTS
-- ========================

--- Channel for telephony signal tones
CHAN_TONE = 0
--- Phone channel 1
CHAN_PHONE1 = 1
--- Phone channel 2
CHAN_PHONE2 = 2
--- Phone channel 3
CHAN_PHONE3 = 3
--- Phone channel 4
CHAN_PHONE4 = 4
--- Phone channel 5
CHAN_PHONE5 = 5
--- Phone channel 6
CHAN_PHONE6 = 6
--- Phone channel 7
CHAN_PHONE7 = 7
--- Phone channel 8
CHAN_PHONE8 = 8
--- Soul channel 1
CHAN_SOUL1 = 9
--- Soul channel 2
CHAN_SOUL2 = 10
--- Soul channel 3
CHAN_SOUL3 = 11
--- Soul channel 4
CHAN_SOUL4 = 12
--- Background channel 1
CHAN_BG1 = 13
--- Background channel 2
CHAN_BG2 = 14
--- Background channel 3
CHAN_BG3 = 15
--- Background channel 4
CHAN_BG4 = 16

-- ========================
-- PHONE STATE CONSTANTS
-- ========================
--- @alias PhoneStateCode integer

--- @type PhoneStateCode
--- Indicates that the phone is currently idle and on the hook.
PHONE_IDLE = 0
--- @type PhoneStateCode
--- Indicates that the phone is off the hook and playing a dial tone.
PHONE_DIAL_TONE = 1
--- @type PhoneStateCode
--- Indicates that the phone is in Post-Dial Delay while the user dials a number.
PHONE_POST_DIAL_DELAY = 2
--- @type PhoneStateCode
--- Indicates that the phone is placing a call and the line is ringing.
PHONE_DIAL_RING = 3
--- @type PhoneStateCode
--- Indicates that the phone is currently in a call.
PHONE_CONNECTED = 4
--- @type PhoneStateCode
--- Indicates that the phone is ringing due to an incoming call.
PHONE_RINGING = 5
--- @type PhoneStateCode
--- Indicates that the phone is playing a busy signal.
PHONE_BUSY_SIGNAL = 6
--- @type PhoneStateCode
--- Indicates that the phone is playing an automated off-hook message.
PHONE_OFF_HOOK_WARN = 7
--- @type PhoneStateCode
--- Indicates that the phone is playing an off-hook signal.
PHONE_OFF_HOOK_SIGNAL = 8

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


if _STUB == true then

    -- ====================================================
    -- ==================== SOUND API =====================
    -- ====================================================
    
    sound = {
        --- Plays a sound on a specific channel.
        ---
        --- Available options:
        --- * `looping: boolean` Make the sound loop (Default: `false`)
        --- * `interrupt: boolean` Stop other sounds on the channel before playing (Default: `true`)
        --- * `speed: number` Multiply the playback speed (Default: `1.0`)
        --- @param path string
        --- @param channel integer
        --- @param opts table
        play = function(path, channel, opts) end,

        --- Returns a boolean indicating whether the specified channel is playing something.
        --- @param channel integer
        --- @return boolean
        is_busy = function(channel) end,

        --- Waits for the specified channel to finish playing.
        --- @param channel integer
        wait = function(channel) end,

        --- Stops playback on a specific channel.
        --- @param channel integer
        stop = function(channel) end,

        --- Stops playback on all channels.
        stop_all = function() end,

        get_channel_volume = function(channel) end,
        set_channel_volume = function(channel, volume) end,
        get_master_volume = function() end,
        set_master_volume = function(volume) end
    }

    -- ====================================================
    -- ==================== PHONE API =====================
    -- ====================================================

    --- @class Phone
    phone = {
        --- Gets the current state code of the phone.
        --- @return PhoneStateCode
        get_state = function() end,
        --- Gets the number that the user has dialed. If the phone is idle, this will return nil.
        --- @return string|nil
        get_dialed_number = function() end
    }

    --- Gets the number of seconds elapsed since the engine was initialized.
    --- @return number
    function get_run_time() end

    --- Pauses execution for the specified number of milliseconds.
    --- @param ms integer
    --- @type function
    function sleep(ms) end

    --- Generates a random number between an inclusive minimum and exclusive maximum.
    --- @param min integer
    --- @param max integer
    --- @type function
    function random_int(min, max) end

    --- Generates a random floating-point number between an inclusive minimum and exclusive maximum.
    --- @param min number
    --- @param max number
    --- @type function
    function random_float(min, max) end

    --- @class CronJob
    local _Cron = {
        --- If the job is ready to run, returns true. Will not return true again until the next time on the schedule.
        --- @return boolean
        ready = function(self) end
    }

    --- Creates a cron schedule from the specified string. Returns nil if the syntax is invalid.
    --- @param schedule string
    --- @return CronJob|nil
    --- @type function
    function cron(schedule) end
end

--- Wrapper around coroutine.yield() specifically for passing service status to the host.
--- @param status ServiceStatusCode
--- @param status_data any
--- @return ServiceDataCode, any
function service_status(status, status_data)
    local data_code, response_data = coroutine.yield(status, status_data)
    return data_code or SERVICE_DATA_NONE, response_data
end

--- Asynchronously waits the specified number of seconds.
--- @param seconds number
function service_wait(seconds)
    local start_time = get_run_time()
    while get_run_time() - start_time < seconds do
        service_status(SERVICE_STATUS_WAITING)
    end
end

--- Forwards the call to the specified number.
--- @param number string
function service_forward_call(number)
    service_status(SERVICE_STATUS_FORWARD, number)
end

--- Starts a call with the user.
--- @return boolean
function service_start_call()
    local data_code = service_status(SERVICE_STATUS_CALL_USER)
    return data_code ~= SERVICE_DATA_USER_BUSY
end

--- Accepts a pending call.
function service_accept_call()
    service_status(SERVICE_STATUS_ACCEPT_CALL)
end

--- Ends the call.
function service_end_call()
    coroutine.yield(SERVICE_STATUS_END_CALL)
end

--- Asynchronously waits for the user to dial a digit, then returns the digit as a string.
--- If a timeout is specified, and no digit is entered within that time, this function returns nil.
--- @param max_seconds number|nil
--- @return string|nil
function service_get_digit(max_seconds)
    local timed = type(max_seconds) == "number" and max_seconds > 0
    if timed then
        while true do
            local data_code, data = service_status(SERVICE_STATUS_REQUEST_DIGIT)
            if data_code == SERVICE_DATA_DIGIT and type(data) == "string" then
                return data
            end
        end
    else
        local start_time = get_run_time()
        while get_run_time() - start_time < max_seconds do
            local data_code, data = service_status(SERVICE_STATUS_REQUEST_DIGIT)
            if data_code == SERVICE_DATA_DIGIT and type(data) == "string" then
                return data
            end
        end
        return nil
    end
end


--- @class PhoneServiceModule
local _PhoneServiceModule = {
    tick = function(self)        
        local status, state = tick_service_state(self)
        return status, state
    end,
    transition = function(self, state)
        if state == self._state then return end
        transition_service_state(self, state)
    end,
    get_state = function(self) return self._state end,
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
        return _PhoneServiceModule[index]
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
        _state_func_tables = {}
    }, M_PhoneServiceModule)
    return module
end

local function empty_func() end

--- Generates a service state machine coroutine.
--- @param service PhoneServiceModule
--- @param new_state ServiceStateCode
--- @param old_state PhoneStateCode
--- @return thread
local function gen_state_coroutine(service, new_state, old_state)
    local state_coroutine = coroutine.create(function()
        local old_func_table = service._state_func_tables[old_state]
        local new_func_table = service._state_func_tables[new_state]

        local on_enter = new_func_table.enter or empty_func
        local on_tick = new_func_table.tick or empty_func
        -- local exit = new_func_table.exit or empty_func
        local prev_on_exit = old_func_table and old_func_table.exit or empty_func

        prev_on_exit(service)
        on_enter(service)
        while true do
            on_tick(service)
            service_status(SERVICE_STATUS_IDLE)
        end
        -- exit(service)
    end)
    return state_coroutine
end

--- Transitions to the specified state on a service.
--- Returns true if the transition was successful; otherwise, returns false.
--- @param service PhoneServiceModule
--- @param state ServiceStateCode
--- @return boolean
function transition_service_state(service, state)
    local prev_state = service._state
    local state_coroutine = gen_state_coroutine(service, state, prev_state)
    
    service._state = state
    service._state_coroutine = state_coroutine
    return state_coroutine ~= nil
end

--- @param service PhoneServiceModule
--- @return ServiceStatusCode, any
function tick_service_state(service)
    local state_coroutine = service._state_coroutine
    if state_coroutine == nil then     
        return SERVICE_STATUS_IDLE, nil 
    end
    
    if coroutine.status(state_coroutine) ~= "dead" then
        local success, status, status_data = coroutine.resume(state_coroutine)
        -- If the coroutine is somehow dead/broken, transition the state
        if not success then
            return SERVICE_STATUS_FINISHED_STATE, nil
        end
        
        -- Check if the state finished, and if so, transition it
        if status == SERVICE_STATUS_FINISHED_STATE and status_data then
            local new_service_state = status_data
            transition_service_state(service, new_service_state)
        end

        -- Return latest status and any associated data
        return status or SERVICE_STATUS_IDLE, status_data
    else
        return SERVICE_STATUS_FINISHED_STATE, nil
    end
end

--- Gets the current state of a service.
--- @param service PhoneServiceModule
--- @return ServiceStateCode
function get_service_state(service)
    return service._state or SERVICE_STATE_IDLE
end

local function print_info()
    print("Cursed API loaded (" .. _VERSION .. ")")
end

print_info()