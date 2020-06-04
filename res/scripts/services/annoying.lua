local S = SERVICE_MODULE("Annoying DTMF Dialer", "*123")

S:set_idle_tick_during(PHONE_IDLE)

local all_digits = { DIGIT_1, DIGIT_2, DIGIT_3, DIGIT_4, DIGIT_5, DIGIT_6, DIGIT_7, DIGIT_8, DIGIT_9, DIGIT_STAR, DIGIT_0, DIGIT_POUND }

S:state(SERVICE_STATE_IDLE, {
    tick = function(self)
        sound.play_dtmf_digit(table.random_choice(all_digits), 0.1, 0.5)
        sound.wait_min(CHAN_TONE, 0.375)
    end
})

return S