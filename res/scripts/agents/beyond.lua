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

module:state(AGENT_STATE_CALL, {
    enter = function(self)
        if self:get_reason() == CALL_REASON_USER_INIT then
            sound.play("handset/pickup*", CHAN_PHONE2)
            if chance(0.5) then
                sound.play("handset/ring_end_" .. rand_int_bias_high(1, 5), CHAN_PHONE3, { volume = 0.25 })
            end
        end

        sound.play("$beyond/amb_waves", CHAN_BG1, { volume = rand_float(0.1, 0.25), speed = rand_float(0.9, 1.1), skip = rand_float(0, 15), looping = true })
        sound.play("$beyond/amb_desert", CHAN_BG2, { volume = rand_float(0.1, 0.25), speed = rand_float(0.9, 1.1), skip = rand_float(0, 15), looping = true })

        agent.wait(rand_float(0, 10))
        while true do
            sound.play_wait("$beyond/lightning_*", CHAN_BG3, { volume = rand_float(0.05, 0.3), speed = rand_float(0.8, 1.1) })
            agent.wait(rand_float(10, 60))
        end
    end
})

return module