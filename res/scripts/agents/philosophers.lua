local module = create_agent("philosophers", "9873763")
module:require_sound_bank('philosophers')

module:state(AGENT_STATE_IDLE, {
    enter = function(self)
        while true do
            agent.wait(60.0 + rand_float(0.0, 60.0))
            if chance(0.01) then
                agent.start_call()
            end
        end
    end,
})

module:state(AGENT_STATE_CALL_IN, {
    tick = function(self)
        agent.wait(rand_float(2.0, 5.0))
        agent.accept_call()
    end
})

module:state(AGENT_STATE_CALL_OUT, {
    enter = function(self)
        agent.wait(rand_float(30.0, 40.0))
        agent.end_call()
    end
})

module:state(AGENT_STATE_CALL, {
    enter = function(self)
        if self:get_call_reason() == CALL_REASON_USER_INIT then
            sound.play("handset/pickup*", CHAN_PHONE2)
            if chance(0.5) then
                sound.play("handset/ring_end_" .. rand_int_bias_high(1, 5), CHAN_PHONE3, { volume = 0.25 })
            end
        end
        sound.play_wait("$philosophers/speeches/*", CHAN_PHONE1)
        agent.end_call()
    end
})

return module