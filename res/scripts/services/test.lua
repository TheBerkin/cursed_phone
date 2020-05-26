local service = PHONE_SERVICE("Test Service", "123")

--[[
    
args:
{
    path (string)
}

--]]
function service.load(args)
    sound.play("ambient/comfort_noise", CHAN_BG1, true)
end

--- Updates the service while not in a call.
--- @return ServiceResponseCode
function service.idle_tick()
    sound.play_next("denise/thinking/*", CHAN_SOUL1)
    sound.wait(CHAN_SOUL1)
    sleep(random_int(1, 7) * 1000)
end

--- Updates the service while in a call.
--- @return ServiceResponseCode
function service.call_tick()
    
end

--- Runs when the user calls the service and the call is pending.
--- Return SERVICE_ACCEPT_CALL to accept the call. Return nil or SERVICE_IDLE to ignore.
--- @return ServiceResponseCode
function service.incoming_call_tick()

end

--- Runs when the service is calling the user and the call is pending.
function service.outgoing_call_tick()

end

function service.on_call_connected()

end

function service.on_call_ended()

end

function service.unload(args)

end

return service