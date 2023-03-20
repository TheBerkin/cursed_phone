local module = new_agent("emergency", "911")
module:set_custom_price(0)
module:set_ringback_enabled(false)

-- Immediately answer calls
module:state(AgentState.CALL_IN, {
    enter = function(self)
        task.accept_call()
    end
})

-- Tell unsuspecting people that this is, in fact, *not* a real phone
module:state(AgentState.CALL, {
    enter = function(self)
        sound.play_wait("intercept/emergency_stub", Channel.PHONE01)
        task.end_call()
    end
})

return module