local S = SERVICE_MODULE("paradise_man", "9871754")
S:require_sound_bank('paradise_man')

S:state(SERVICE_STATE_IDLE, {
    enter = function(self)
        while true do
            service.wait(60.0 + rand_float(0.0, 60.0))
            if chance(0.01) then
                service.start_call()
            end
        end
    end,
})

S:state(SERVICE_STATE_CALL_IN, {
    tick = function(self)
        service.wait(rand_float(3.0, 5.0))
        service.accept_call()
    end
})

S:state(SERVICE_STATE_CALL_OUT, {
    enter = function(self)
        service.wait(rand_float(30.0, 40.0))
        service.end_call()
    end
})

S:state(SERVICE_STATE_CALL, {
    enter = function(self)
        sound.play_wait("$paradise_man/speech", CHAN_PHONE1)
        service.end_call()
    end
})

return S