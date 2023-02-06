local module = create_agent('toll', nil, AGENT_ROLE_TOLLMASTER)

local NAG_TONE_VOLUME = 0.5

module:state(AGENT_STATE_IDLE, {
    tick = function(self)
        if toll.is_awaiting_deposit() then
            print("Tollmaster: Insert " .. toll.current_call_rate() .. "¢ now!")
            sound.play_special_info_tone(SpecialInfoTone.INEFFECTIVE)
            sound.wait(CHAN_SIGIN)
            sound.play("toll/nag_start_call", CHAN_BG1)
            sound.wait_min(CHAN_BG1, 10.0)
        elseif toll.is_time_low() then
            print("Tollmaster: Insert " .. toll.current_call_rate() .. "¢ now!")
            sound.play_wait("toll/nag_tone", CHAN_BG2, { volume = NAG_TONE_VOLUME })
            sound.play("toll/nag_extend_call", CHAN_BG1)
            sound.wait_min(CHAN_BG1, 10.0)
        end
        agent.wait(1)
    end
})

return module