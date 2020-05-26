local service = PHONE_SERVICE("Test Service", nil)

--[[
    
args:
{
    script_path (string)
}

--]]
function service.load(args)
    sound.play("denise/thinking/hello", CHAN_SOUL1, false)
end

--- Updates the service while not in a call.
function service.idle_tick()

end

--- @return ServiceCode
function service.call_tick()
    sound.play("denise/thinking/go", CHAN_SOUL1, false)
end

--- Runs when the user calls the service and the call is pending.
--- Return SERVICE_ACCEPT_CALL to accept the call. Return nil or SERVICE_IDLE to ignore.
--- @return ServiceCode
function service.incoming_call_tick()

end

--- Runs when the service is calling the user and the call is pending.
function service.outgoing_call_tick()

end

function service.on_call_connected()

end

function service.on_call_end()

end

function service.yeet(args)

end

return service