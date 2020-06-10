local S = SERVICE_MODULE("denise")

S:set_idle_tick_during(PHONE_STATE_IDLE, PHONE_STATE_DIAL_TONE)
-- S:suspend()

local msg_handlers = {
    ["mark_start"] = function(self, msg_data)
        -- Mark call starts
        service.wait(rand_float(0.8, 2.0))
        if not sound.is_busy(CHAN_SOUL1) then
            sound.play("denise/reactions/mark_greet/*", CHAN_SOUL1, { interrupt = false })
        end
    end,
    ["mark_talk"] = function(self, msg_data)
        -- Mark says something
        service.wait(rand_float(1.75, 2.5))
        if not sound.is_busy(CHAN_SOUL1) then
            sound.play("denise/reactions/mark_plead/*", CHAN_SOUL1, { interrupt = false })
        end
    end,
    ["mark_end"] = function(self, msg_data)
        -- Mark call ends
        service.wait(rand_float(1.0, 3.0))
        if not sound.is_busy(CHAN_SOUL1) then
            sound.play("denise/reactions/mark_post/*", CHAN_SOUL1, { interrupt = false })
        end
    end
}

-- args.path: path to this script file
function S.load(args)    
end

S:state(SERVICE_STATE_IDLE, {
    enter = function(self)
        --sound.play("ambient/static", CHAN_BG1, { looping = true, volume = 0.6 })
        --service.wait(rand_float(0, 1))
        --sound.play_wait("denise/seeing/hi", CHAN_SOUL1)
    end,

    tick = function(self)
        --service.wait(rand_float(2, 12))
        --sound.play_wait("denise/thinking/*", CHAN_SOUL1, { interrupt = false })
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