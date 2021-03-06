local S = SERVICE_MODULE("operator", "0")
S:set_custom_price(0)

S:state(SERVICE_STATE_CALL, {
    enter = function(self)
        print("Operator: call started")
    end,
    tick = function(self)
        -- TODO: Implement Operator
        while true do
            local digit = service.read_digit()
            print("Operator: Got digit '" .. digit .. "'")
        end
    end,
    exit = function(self)
        print("Operator: ending call")
    end
})


S:state(SERVICE_STATE_CALL_IN, {
    tick = function(self)
        service.wait(rand_float(1.0, 3.0))
        service.accept_call()
    end
})

return S