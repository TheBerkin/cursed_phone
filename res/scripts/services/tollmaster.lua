local S = SERVICE_MODULE('toll', nil, SERVICE_ROLE_TOLLMASTER)

local TONE_VOLUME = 0.5

S:state(SERVICE_STATE_IDLE, {
    tick = function(self)
        if toll.is_awaiting_deposit() then
            print("Tollmaster: Insert " .. toll.current_call_rate() .. "¢ now!")
            sound.play("toll/nag_start_call", CHAN_BG1)
            sound.wait_min(CHAN_SOUL4, 10.0)
        elseif toll.is_time_low() then
            print("Tollmaster: Insert " .. toll.current_call_rate() .. "¢ now!")
            sound.play_wait("toll/nag_tone", CHAN_BG2, { volume = TONE_VOLUME })
            sound.play("toll/nag_extend_call", CHAN_BG1)
            sound.wait_min(CHAN_SOUL4, 10.0)
        end
        service.wait(1)
    end
})

return S