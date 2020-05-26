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
-- SERVICE CODE CONSTANTS
-- ========================
--- @alias ServiceResponseCode integer

--- @type ServiceResponseCode
--- Indicates that a service performed no action.
SERVICE_IDLE = 0
--- @type ServiceResponseCode
--- Indicates that a service is accepting an incoming call.
SERVICE_ACCEPT_CALL = 1
--- @type ServiceResponseCode
--- Indicates that a service is hanging up.
SERVICE_END_CALL = 2

-- ========================
-- SOUND API
-- ========================

if _STUB == true then
    
    sound = {
        --- Plays a sound on a specific channel.
        --- @param path string
        --- @param channel integer
        --- @param looping boolean
        play = function(path, channel, looping) end,

        --- Plays a sound on a specific channel and waits for it to finish.
        --- @param path string
        --- @param channel integer
        --- @param looping boolean
        play_wait = function(path, channel, looping) end,

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

    -- ========================
    -- PHONE API
    -- ========================

    --- @class Phone
    phone = phone or {
        get_state = function() end,
        read_digit = function() end,
        register_callee = function(phone_number) end,
        --- VVVVVVVVVVV
        vibrate = function(power, time_ms) end
    }

    --- Pauses execution for the specified number of milliseconds.
    --- @param ms integer
    sleep = sleep or function(ms) end

    --- Vibrates.
    --- @param power number|"1"
    --- @param time_ms integer|"1000"
    vibrate = vibrate or function(power, time_ms) end

    --- Subscribes a function to an event type.
    --- @param event_name string @The name of the event.
    --- @param handler function @The function called when the event occurs.
    add_event_listener = add_event_listener or function(event_name, handler) end

    --- Unsubscribes a function from an event type.
    --- @param event_name string @The name of the event.
    --- @param handler function @The function called when the event occurs.
    remove_event_listener = remove_event_listener or function(event_name, handler) end


end

--- Returns an empty phone service module.
--- @param name string @The display name of the phone service
--- @param phone_number string | nil @The number associated with the phone service
--- @return PhoneServiceModule
function PHONE_SERVICE(name, phone_number)
    --- @class PhoneServiceModule
    local module = {
        _name = name,
        _phone_number = phone_number
    }

    return module
end

-- ========================
-- INTERNAL FUNCTIONS
-- ========================