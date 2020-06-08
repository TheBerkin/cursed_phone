local S = SERVICE_MODULE("phantoms", "3763987")
S:require_sound_bank('phantoms')

S:state(SERVICE_STATE_CALL, {
    enter = function(self)
        sound.play("handset/pickup*", CHAN_PHONE2)
        sound.play_wait("$phantoms/speech", CHAN_PHONE1)
        service.end_call()
    end,
})


S:state(SERVICE_STATE_CALL_IN, {
    tick = function(self)
        service.wait(random_float(2.0, 5.0))
        service.accept_call()
    end
})

return S