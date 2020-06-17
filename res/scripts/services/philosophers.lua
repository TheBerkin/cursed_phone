local S = SERVICE_MODULE("philosophers", "9873763")
S:require_sound_bank('philosophers')

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
        service.wait(rand_float(2.0, 5.0))
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
        if S:get_reason() == CALL_REASON_USER_INIT then
            sound.play("handset/pickup*", CHAN_PHONE2)
            if chance(0.5) then
                sound.play("handset/ring_end_" .. rand_int_bias_high(1, 5), CHAN_PHONE3, { volume = 0.25 })
            end
        end
        sound.play_wait("$philosophers/speeches/*", CHAN_PHONE1, { volume = 0.9 })
        service.end_call()
    end
})

return S