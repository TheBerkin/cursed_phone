local S = SERVICE_MODULE("Intercept Service", "A", SERVICE_ROLE_INTERCEPT)

S:state(SERVICE_STATE_CALL, {
    enter = function(self)
        print("Intercept: call started")
    end,
    tick = function(self)
        sound.play_wait("intercept/intercept_message_01", CHAN_PHONE1)
        service.end_call()
    end,
    exit = function(self)
        print("Intercept: ending call")
    end
})


S:state(SERVICE_STATE_CALL_IN, {
    enter = function(self)
        service.accept_call()
    end
})

return S