--- @meta

--- Provides information about and exposes functionality specific to the phone state and physical interface.
phone = {}

--- Returns the internal ID of the last agent who called the phone, or `nil` if nobody called yet.
--- @return integer?
function phone.last_caller_id() end

--- Gets the last number called by the user.
--- @return string
function phone.last_dialed_number() end

--- Gets the number dialed by the user to place the current call.
--- @return string?
function phone.call_dialed_number() end

--- Forces the phone to dial the specified digit(s).
function phone.dial(digits) end

--- Returns a boolean value indicating whether the host phone is registered as a rotary phone.
--- @return boolean
function phone.is_rotary() end

--- Returns a boolean value indicating whether the rotary dial (if available) is in the resting position, or `nil` if the phone isn't a rotary phone.
--- @return boolean?
function phone.is_rotary_dial_resting() end

--- Returns a boolean value indicating whether the phone is currently on-hook.
--- @return boolean
function phone.is_on_hook() end

--- Rings the phone.
--- @param pattern string | RingPattern @ The pattern to send to the ringer.
function phone.ring(pattern) end

--- Stops all ringing.
function phone.stop_ringing() end

--- @param expr string
--- @return boolean, RingPattern?
function phone.compile_ring_pattern(expr) end

--- Sets the locked status of the switchhook.
--- When the switchhook is locked, new switchhook inputs will be ignored.
--- @param is_locked boolean
function phone.set_switchhook_locked(is_locked) end

--- Returns the locked status of the switchhook.
--- @return boolean
function phone.is_switchhook_locked() end