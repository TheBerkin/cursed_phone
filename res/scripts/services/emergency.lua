local S = SERVICE_MODULE("emergency", "911")
S:set_custom_price(0)
S:set_ringback_enabled(false)

-- Immediately answer calls
S:state(SERVICE_STATE_CALL_IN, {
    enter = function(self)
        service.accept_call()
    end
})

-- Tell unsuspecting people that this is, in fact, *not* a real phone
S:state(SERVICE_STATE_CALL, {
    enter = function(self)
        sound.play_wait("intercept/emergency_stub", CHAN_PHONE1)
        service.end_call()
    end
})

return S