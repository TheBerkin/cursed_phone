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
--- Call was placed because the originally dialed number was redirected.
CALL_REASON_NUMBER_REDIRECTED = 2
--- @type CallReason
--- Call was placed by the user.
CALL_REASON_USER_INIT = 3
--- @type CallReason
--- Call was placed by an agent.
CALL_REASON_AGENT_INIT = 4


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

--- @enum SpecialInfoTone
--- Defines Special Information Tone (SIT) types.
SpecialInfoTone = {
    --- Unassigned N11 code, CLASS code, or prefix.
    VACANT_CODE = 0,
    --- Incomplete digits, internal office or feature failure (local office).
    REORDER_INTRA = 1,
    --- Call failure, no wink or partial digits received (distant office).
    REORDER_INTER = 2,
    --- All circuits busy (local office).
    NO_CIRCUIT_INTRA = 3,
    --- All circuits busy (distant office).
    NO_CIRCUIT_INTER = 4,
    --- Number changed or disconnected.
    INTERCEPT = 5,
    --- General misdialing, coin deposit required or other failure.
    INEFFECTIVE = 6,
    --- Reserved for future use.
    RESERVED = 7
}

if not phone then
    --- Provides information about and exposes functionality specific to the phone state and physical interface.
    phone = {}

    --- Returns the internal ID of the last agent who called the phone, or `nil` if nobody called yet.
    --- @return integer|nil
    function phone.last_caller_id() return nil end

    --- Returns a boolean value indicating whether the host phone is registered as a rotary phone.
    --- @return boolean
    function phone.is_rotary() return false end
