--[[

    /==========================================================================\
    |========================= CURSED PHONE API FILE ==========================|
    |==========================================================================|
    | This script is required by the engine in order to function properly.     |
    | Unless you are making changes to the engine, do not modify this file.    |
    \==========================================================================/
    
]]

-- ====================================================
-- ==================== PHONE API =====================
-- ====================================================

--- @enum CallReason
--- Defines reason codes for phone calls.
CallReason = {
    --- No call reason given.
    NONE = 0,
    --- Call was placed because of an off-hook timeout.
    OFF_HOOK = 1,
    --- Call was placed because the originally dialed number was redirected.
    REDIRECTED = 2,
    --- Call was placed by the user.
    USER_INIT = 3,
    --- Call was placed by an agent.
    AGENT_INIT = 4
}


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

    --- Gets the last number called by the user.
    --- @return string
    function phone.last_dialed_number() return nil end

    --- Forces the phone to dial the specified digit(s).
    function phone.dial(digits) end

    --- Returns a boolean value indicating whether the host phone is registered as a rotary phone.
    --- @return boolean
    function phone.is_rotary() return false end

    --- Returns a boolean value indicating whether the rotary dial (if available) is in the resting position, or `nil` if the phone isn't a rotary phone.
    --- @return boolean|nil
    function phone.is_rotary_dial_resting() return nil end

    --- Returns a boolean value indicating whether the phone is currently on-hook.
    --- @return boolean
    function phone.is_on_hook() return false end

    --- Rings the phone.
    --- @param pattern string | RingPattern @ The pattern to send to the ringer.
    function phone.ring(pattern) end

    --- Stops all ringing.
    function phone.stop_ringing() end

    --- @param expr string
    --- @return boolean, RingPattern?
    function phone.compile_ring_pattern(expr) return false, nil end
end