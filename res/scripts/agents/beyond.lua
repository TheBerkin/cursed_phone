local module = create_agent("beyond", "2402")

module:require_sound_bank("beyond")

module:state(AGENT_STATE_CALL_IN, {
    enter = function(self)
        agent.wait(rand_float(2, 8))
        agent.accept_call()
    end
})

--- @async
local function do_lightning()
    agent.wait(rand_float(0, 5))
    sound.play_wait("$beyond/lightning_*", Channel.PHONE07, {
        volume = rand_float(0.05, 0.4),
        speed = rand_float(0.8, 1.1),
        skip = rand_float(0, 10)
    })
    while true do
        agent.wait(rand_float(5, 50))
        sound.play_wait("$beyond/lightning_*", Channel.PHONE07, {
            volume = rand_float(0.05, 0.4),
            speed = rand_float(0.8, 1.1)
        })
    end
end

--- @async
local function do_seagulls()
    agent.wait(rand_float(0, 5))
    sound.play_wait("$beyond/seagulls_*", Channel.PHONE06, {
        volume = rand_float(0.05, 0.1),
        speed = rand_float(0.9, 1.2),
        skip = rand_float(0, 10),
        fadein = 2
    })
    while true do
        agent.wait(rand_float(5, 40))
        sound.play_wait("$beyond/seagulls_*", Channel.PHONE06, {
            volume = rand_float(0.05, 0.1),
            speed = rand_float(0.9, 1.2),
            fadein = 2
        })
    end
end

--- @async
local function do_chimes()
    agent.wait(rand_float(0, 5))
    sound.play_wait("$beyond/chimes_*", Channel.PHONE08, {
        volume = rand_float(0.01, 0.025),
        speed = rand_float(0.9, 1.05),
        skip = rand_float(0, 4),
        fadein = 3
    })
    while true do
        agent.wait(rand_float(5, 20))
        sound.play_wait("$beyond/chimes_*", Channel.PHONE08, {
            volume = rand_float(0.01, 0.025),
            speed = rand_float(0.9, 1.05),
            skip = rand_float(0, 4),
            fadein = 3
        })
    end
end

module:state(AGENT_STATE_CALL, {
    enter = function(self)
        if self:get_reason() == CALL_REASON_USER_INIT then
            sound.play("handset/pickup*", Channel.PHONE02, { skip = rand_float(0, 0.3) })
            if chance(0.5) then
                sound.play("handset/ring_end_" .. rand_int_bias_high(1, 5), Channel.PHONE03, { volume = 0.25 })
            end
        end

        sound.play("$beyond/amb_waves", Channel.PHONE03, {
            volume = rand_float(0.1, 0.25),
            speed = rand_float(0.9, 1.1),
            skip = rand_float(0, 15),
            looping = true
        })
        sound.play("$beyond/amb_desert", Channel.PHONE04, {
            volume = rand_float(0.1, 0.25),
            speed = rand_float(0.9, 1.1),
            skip = rand_float(0, 15),
            looping = true
        })
        sound.play("$beyond/amb_crows", Channel.PHONE05, {
            volume = rand_float(0.02, 0.04),
            speed = rand_float(0.9, 1.1),
            skip = rand_float(0, 15),
            looping = true
        })

        agent.multi_task(do_lightning, do_seagulls, do_chimes)
    end
})

return module