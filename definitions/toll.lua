--- @meta

--- @class PayphoneLib
toll = {}

--- Returns `true` if the remaining call time credit is below the warning threshold.
--- @return boolean
function toll.is_time_low() end

--- Returns number of seconds remaining of call time credit.
--- @return number
function toll.time_left() end

--- Returns the rate (in lowest denomination) of the current call.
--- If no call is active, returns the standard rate.
--- @return integer
function toll.current_call_rate() end

--- Returns `true` if the current call is free, or `false` if it has a nonzero rate or there is no active call.
function toll.is_current_call_free() end

--- Returns `true` if the phone is currently waiting for a deposit to place a call.
function toll.is_awaiting_deposit() end