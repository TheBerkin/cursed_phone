local S = SERVICE_MODULE("Denise", "123")

-- args.path: path to this script file
function S.load(args)    
end

S:state(SERVICE_STATE_IDLE, {
    enter = function(self)
        sound.set_channel_volume(CHAN_BG1, 0.6)
        sound.play("ambient/static", CHAN_BG1, { looping = true })
    end,

    tick = function(self)
        sound.play("denise/thinking/*", CHAN_SOUL1, {
            interrupt = false
        })
        while sound.is_busy(CHAN_SOUL1) do
            service_status(SERVICE_STATUS_IDLE)
        end
        service_wait(random_float(1, 10))
    end,

    exit = function(self)

    end
})


function S.unload(args)
end

return S