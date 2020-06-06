local S = SERVICE_MODULE("operator", "0")

S:state(SERVICE_STATE_CALL, {
    enter = function(self)
        print("Operator: call started")
    end,
    tick = function(self)
        -- TODO
    end,
    exit = function(self)
        print("Operator: ending call")
    end
})


S:state(SERVICE_STATE_CALL_IN, {
    tick = function(self)
        service.wait(random_float(1.0, 5.0))
        service.accept_call()
    end
})

return S