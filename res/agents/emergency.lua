local agent = AgentModule("emergency", "911")
agent:set_custom_price(0)
agent:set_ringback_enabled(false)

-- Immediately answer calls
agent:state(AgentState.CALL_IN, {
    enter = function(self)
        task.accept_call()
    end
})

-- Tell unsuspecting people that this is, in fact, *not* a real phone
agent:state(AgentState.CALL, {
    enter = function(self)
        sound.play_wait("intercept/emergency_stub", Channel.PHONE01)
        task.end_call()
    end
})

return agent