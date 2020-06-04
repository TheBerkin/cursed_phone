local S = SERVICE_MODULE("Denise", "*666")

S:set_idle_tick_during(PHONE_IDLE, PHONE_DIAL_TONE)

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

    exit = function(self)

    end
})

function S.unload(args)
end

return S