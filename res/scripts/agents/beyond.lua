local module = AGENT_MODULE("beyond", "2402")

module:require_sound_bank("beyond")

module:state(AGENT_STATE_IDLE, {
    
})

module:state(AGENT_STATE_CALL_IN, {
    enter = function(self)
        agent.wait(rand_float(2, 8))
        agent.accept_call()
    end
})

local function do_lightning()
    agent.wait(rand_float(0, 5))
    sound.play_wait("$beyond/lightning_*", CHAN_PHONE5, { volume = rand_float(0.05, 0.4), speed = rand_float(0.8, 1.1), skip = rand_float(0, 10) })
    while true do
        agent.wait(rand_float(5, 50))
        sound.play_wait("$beyond/lightning_*", CHAN_PHONE5, { volume = rand_float(0.05, 0.4), speed = rand_float(0.8, 1.1) })
    end
end

local function do_seagulls()
    agent.wait(rand_float(0, 5))
    sound.play_wait("$beyond/seagulls_*", CHAN_PHONE6, { volume = rand_float(0.05, 0.1), speed = rand_float(0.9, 1.2), skip = rand_float(0, 10), fadein = 2 })
    while true do
        agent.wait(rand_float(5, 40))
        sound.play_wait("$beyond/seagulls_*", CHAN_PHONE6, { volume = rand_float(0.05, 0.1), speed = rand_float(0.9, 1.2), fadein = 2 })
    end
end

module:state(AGENT_STATE_CALL, {
    enter = function(self)
        if self:get_reason() == CALL_REASON_USER_INIT then
            sound.play("handset/pickup*", CHAN_PHONE2, { skip = rand_float(0, 0.3) })
            if chance(0.5) then
                sound.play("handset/ring_end_" .. rand_int_bias_high(1, 5), CHAN_PHONE3, { volume = 0.25 })
            end
        end

        sound.play("$beyond/amb_waves", CHAN_PHONE3, { volume = rand_float(0.1, 0.25), speed = rand_float(0.9, 1.1), skip = rand_float(0, 15), looping = true })
        sound.play("$beyond/amb_desert", CHAN_PHONE4, { volume = rand_float(0.1, 0.25), speed = rand_float(0.9, 1.1), skip = rand_float(0, 15), looping = true })

        agent.multi_task(do_lightning, do_seagulls)
    end
})

return module