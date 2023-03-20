local agent = AgentModule("beyond", "2402")

agent:require_sound_bank("beyond")

agent:state(AgentState.CALL_IN, {
    enter = function(self)
        task.wait(randf(2, 8))
        task.accept_call()
    end
})

--- @async
local function do_lightning()
    task.wait(randf(0, 5))
    sound.play_wait("$beyond/lightning_*", Channel.PHONE07, {
        volume = randf(0.05, 0.4),
        speed = randf(0.8, 1.1),
        skip = randf(0, 10)
    })
    while true do
        task.wait(randf(5, 50))
        sound.play_wait("$beyond/lightning_*", Channel.PHONE07, {
            volume = randf(0.05, 0.4),
            speed = randf(0.8, 1.1)
        })
    end
end

--- @async
local function do_seagulls()
    task.wait(randf(0, 5))
    sound.play_wait("$beyond/seagulls_*", Channel.PHONE06, {
        volume = randf(0.05, 0.1),
        speed = randf(0.9, 1.2),
        skip = randf(0, 10),
        fadein = 2
    })
    while true do
        task.wait(randf(5, 40))
        sound.play_wait("$beyond/seagulls_*", Channel.PHONE06, {
            volume = randf(0.05, 0.1),
            speed = randf(0.9, 1.2),
            fadein = 2
        })
    end
end

--- @async
local function do_chimes()
    task.wait(randf(0, 5))
    sound.play_wait("$beyond/chimes_*", Channel.PHONE08, {
        volume = randf(0.01, 0.025),
        speed = randf(0.9, 1.05),
        skip = randf(0, 4),
        fadein = 3
    })
    while true do
        task.wait(randf(5, 20))
        sound.play_wait("$beyond/chimes_*", Channel.PHONE08, {
            volume = randf(0.01, 0.025),
            speed = randf(0.9, 1.05),
            skip = randf(0, 4),
            fadein = 3
        })
    end
end

agent:state(AgentState.CALL, {
    enter = function(self)
        if self:get_call_reason() == CallReason.USER_INIT then
            sound.play("handset/pickup*", Channel.PHONE01, { skip = randf(0, 0.3) })
            if maybe(0.5) then
                sound.play("handset/ring_end", Channel.PHONE02, { volume = 0.25, skip = 'random' })
            end
        end

        sound.play("$beyond/amb_waves", Channel.PHONE03, {
            volume = randf(0.1, 0.25),
            speed = randf(0.9, 1.1),
            skip = randf(0, 15),
            looping = true
        })
        sound.play("$beyond/amb_desert", Channel.PHONE04, {
            volume = randf(0.1, 0.25),
            speed = randf(0.9, 1.1),
            skip = randf(0, 15),
            looping = true
        })
        sound.play("$beyond/amb_crows", Channel.PHONE05, {
            volume = randf(0.02, 0.04),
            speed = randf(0.9, 1.1),
            skip = randf(0, 15),
            looping = true
        })

        task.parallel(do_lightning, do_seagulls, do_chimes)
    end
})

return agent