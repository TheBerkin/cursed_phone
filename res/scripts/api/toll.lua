--[[

    /==========================================================================\
    |========================= CURSED PHONE API FILE ==========================|
    |==========================================================================|
    | This script is required by the engine in order to function properly.     |
    | Unless you are making changes to the engine, do not modify this file.    |
    \==========================================================================/
    
]]

if not toll then
    toll = {}

    --- Returns `true` if the remaining call time credit is below the warning threshold.
    --- @return boolean
    function toll.is_time_low() return false end

    --- Returns number of seconds remaining of call time credit.
    --- @return number
    function toll.time_left() return 0 end

    --- Returns the rate (in lowest denomination) of the current call.
    --- If no call is active, returns the standard rate.
    --- @return integer
    function toll.current_call_rate() return 0 end

    --- Returns `true` if the current call is free, or `false` if it has a nonzero rate or there is no active call.
    function toll.is_current_call_free() end

    --- Returns `true` if the phone is currently waiting for a deposit to place a call.
    function toll.is_awaiting_deposit() end
end