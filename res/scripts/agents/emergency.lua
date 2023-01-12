local S = AGENT_MODULE("emergency", "911")
S:set_custom_price(0)
S:set_ringback_enabled(false)

-- Immediately answer calls
S:state(AGENT_STATE_CALL_IN, {
    enter = function(self)
        agent.accept_call()
    end
})

-- Tell unsuspecting people that this is, in fact, *not* a real phone
S:state(AGENT_STATE_CALL, {
    enter = function(self)
        sound.play_wait("intercept/emergency_stub", CHAN_PHONE1)
        agent.end_call()
    end
})

return S