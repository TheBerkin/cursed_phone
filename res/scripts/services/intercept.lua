local S = SERVICE_MODULE("Intercept Service", "A", SERVICE_ROLE_INTERCEPT)

S:state(SERVICE_STATE_CALL, {
    enter = function(self)
        print("Intercept: call started")
    end,
    tick = function(self)
        sound.play("intercept/intercept_message_01", CHAN_PHONE1)
        while sound.is_busy(CHAN_PHONE1) do
            service_status(SERVICE_STATUS_WAITING)
        end
        service_end_call()
    end,
    exit = function(self)
        print("Intercept: ending call")
    end
})


S:state(SERVICE_STATE_CALL_IN, {
    enter = function(self)
        service_accept_call()
    end
})

return S