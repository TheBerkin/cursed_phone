local S = SERVICE_MODULE("mark")

S:set_idle_tick_during(PHONE_STATE_IDLE, PHONE_STATE_DIAL_TONE)

-- args.path: path to this script file
function S.load(args)    
end

S:state(SERVICE_STATE_IDLE, {
    tick = function(self)
        -- TODO: Implement Mark
    end,
})

function S.unload(args)
end

return S