local S = SERVICE_MODULE("Intercept Service", "A", SERVICE_ROLE_INTERCEPT)

--[[
    
args:
{
    path (string)
}

--]]
function S.load(args)
end

--- Updates the service while not in a call.
--- @return ServiceStatusCode
function S.idle_tick()
end

--- Updates the service while in a call.
--- @return ServiceStatusCode
function S.call_tick()
end

--- Runs when the user calls the service and the call is pending.
--- Return SERVICE_ACCEPT_CALL to accept the call. Return nil or SERVICE_IDLE to ignore.
--- @return ServiceStatusCode
function S.incoming_call_tick()
end

--- Runs when the service is calling the user and the call is pending.
--- @return ServiceStatusCode
function S.outgoing_call_tick()
end

function S.on_call_connected()

end

function S.on_call_ended()

end

function S.unload(args)

end

return S