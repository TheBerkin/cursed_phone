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

--- @alias PhoneDigit integer

--- @type PhoneDigit
--- The digit '1'.
DIGIT_1 = 49
--- @type PhoneDigit
--- The digit '2'.
DIGIT_2 = 50
--- @type PhoneDigit
--- The digit '3'.
DIGIT_3 = 51
--- @type PhoneDigit
--- The digit 'A'.
DIGIT_A = 65
--- @type PhoneDigit
--- The digit '4'.
DIGIT_4 = 52
--- @type PhoneDigit
--- The digit '5'.
DIGIT_5 = 53
--- @type PhoneDigit
--- The digit '6'.
DIGIT_6 = 54
--- @type PhoneDigit
--- The digit 'B'.
DIGIT_B = 66
--- @type PhoneDigit
--- The digit '7'.
DIGIT_7 = 55
--- @type PhoneDigit
--- The digit '8'.
DIGIT_8 = 56
--- @type PhoneDigit
--- The digit '9'.
DIGIT_9 = 57
--- @type PhoneDigit
--- The digit 'C'.
DIGIT_C = 67
--- @type PhoneDigit
--- The digit '*'.
DIGIT_STAR = 42
--- @type PhoneDigit
--- The digit '0'.
DIGIT_0 = 48
--- @type PhoneDigit
--- The digit '#'.
DIGIT_POUND = 35

--- @alias CallReason integer

--- @type CallReason
--- No call reason given.
CALL_REASON_NONE = 0
--- @type CallReason
--- Call was placed because of an off-hook timeout.
CALL_REASON_OFF_HOOK = 1
--- @type CallReason
--- Call was placed because the originally dialed number was disconnected.
CALL_REASON_NUMBER_DISCONNECTED = 2
--- @type CallReason
--- Call was placed by the user.
CALL_REASON_USER_INIT = 3
--- @type CallReason
--- Call was placed by a service.
CALL_REASON_SERVICE_INIT = 4


--- @alias PhoneStateCode integer

--- @type PhoneStateCode
--- Indicates that the phone is currently idle and on the hook.
PHONE_STATE_IDLE = 0
--- @type PhoneStateCode
--- Indicates that the phone is off the hook and playing a dial tone.
PHONE_STATE_DIAL_TONE = 1
--- @type PhoneStateCode
--- Indicates that the phone is in Post-Dial Delay while the user dials a number.
PHONE_STATE_PDD = 2
--- @type PhoneStateCode
--- Indicates that the phone is placing a call and the line is ringing.
PHONE_STATE_RINGBACK = 3
--- @type PhoneStateCode
--- Indicates that the phone is currently in a call.
PHONE_STATE_CONNECTED = 4
--- @type PhoneStateCode
--- Indicates that the phone is ringing due to an incoming call.
PHONE_STATE_RINGING = 5
--- @type PhoneStateCode
--- Indicates that the phone is playing a busy signal (for varying reasons).
PHONE_STATE_BUSY_TONE = 6

-- NATIVE PHONE FUNCTIONS
NATIVE_API(function()
    phone = {}

    function phone.vibrate(power, time) end

    function phone.vibrate_set(power) end

    function phone.vibrate_stop() end
end)