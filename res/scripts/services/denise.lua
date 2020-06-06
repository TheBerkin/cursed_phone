local S = SERVICE_MODULE("denise")

S:set_idle_tick_during(PHONE_STATE_IDLE, PHONE_STATE_DIAL_TONE)

local msg_handlers = {
    ["mark_start"] = function(self, msg_data)
        -- Mark call starts
    end,
    ["mark_talk"] = function(self, msg_data)
        -- Mark says something
    end,
    ["mark_end"] = function(self, msg_data)
        -- Mark call ends
    end
}

-- args.path: path to this script file
function S.load(args)    
end

S:state(SERVICE_STATE_IDLE, {
    enter = function(self)
        sound.play("ambient/static", CHAN_BG1, { looping = true, volume = 0.6 })
        service.wait(random_float(0, 1))
        sound.play_wait("denise/seeing/hi", CHAN_SOUL1)
    end,

    tick = function(self)
        service.wait(random_float(2, 12))
        sound.play_wait("denise/thinking/*", CHAN_SOUL1, { interrupt = false })
    end,

    message = function(self, sender, msg_type, msg_data)
        local handler = msg_handlers[msg_type]
        if type(handler) == 'function' then
            handler(self, msg_data)
        end
    end,

    exit = function(self)

    end
})

function S.unload(args)
end

return S