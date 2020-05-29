local S = SERVICE_MODULE("Denise", "123")

--[[
    
args:
{
    path (string)
}

--]]
function S.load(args)
    sound.set_channel_volume(CHAN_BG1, 0.6)
    sound.play("ambient/static", CHAN_BG1, { looping = true })
end

function S.idle_enter()

end

--- Updates the service while not in a call.
--- @return ServiceStatusCode
function S.idle_tick()
    sound.play("denise/thinking/*", CHAN_SOUL1, {
        interrupt = false
    })
    while sound.is_busy(CHAN_SOUL1) do
        service_status(SERVICE_STATUS_IDLE)
    end
    service_wait(random_float(1, 10))
end

function S.idle_exit()

end

function S.call_enter()

end

--- Updates the service while in a call.
--- @return ServiceStatusCode
function S.call_tick()
    
end

function S.call_exit()

end

--- Runs when the user calls the service and the call is pending.
--- Return SERVICE_ACCEPT_CALL to accept the call. Return nil or SERVICE_IDLE to ignore.
--- @return ServiceStatusCode
function S.incoming_call_tick()

end

--- Runs when the service is calling the user and the call is pending.
function S.outgoing_call_tick()

end

function S.unload(args)

end

return S