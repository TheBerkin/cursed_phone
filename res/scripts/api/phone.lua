--[[

    /==========================================================================\
    |========================= CURSED PHONE API FILE ==========================|
    |==========================================================================|
    | This script is required by phone services in order to function properly. |
    | Unless you are making changes to the engine, do not modify this file.    |
    \==========================================================================/
    
]]

-- ====================================================
-- ==================== PHONE API =====================
-- ====================================================

DIGIT_1 = 49
DIGIT_2 = 50
DIGIT_3 = 51
DIGIT_A = 65
DIGIT_4 = 52
DIGIT_5 = 53
DIGIT_6 = 54
DIGIT_B = 66
DIGIT_7 = 55
DIGIT_8 = 56
DIGIT_9 = 57
DIGIT_C = 67
DIGIT_STAR = 42
DIGIT_0 = 48
DIGIT_POUND = 35

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

-- NATIVE PHONE FUNCTIONS
NATIVE_API(function()
    phone = {}

    --- Gets the current state code of the phone.
    --- @return PhoneStateCode
    function phone.get_state() end
    --- Gets the number that the user has dialed. If the phone is idle, this will return nil.
    --- @return string|nil
    function phone.get_dialed_number() end

    function phone.vibrate(power, time) end

    function phone.vibrate_set(power) end
    
    function phone.vibrate_stop() end
end)
