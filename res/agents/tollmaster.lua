local agent = AgentModule('toll', nil, AgentRole.TOLLMASTER)

local NAG_TONE_VOLUME = 0.5

agent:state(AgentState.IDLE, {
    tick = function(self)
        if toll.is_awaiting_deposit() then
            print("Tollmaster: Insert " .. toll.current_call_rate() .. "¢ now!")
            sound.play_special_info_tone(SpecialInfoTone.INEFFECTIVE)
            sound.wait(Channel.SIG_IN)
            sound.play("toll/nag_start_call", Channel.BG01)
            sound.wait_min(Channel.BG01, 10.0)
        elseif toll.is_time_low() then
            print("Tollmaster: Insert " .. toll.current_call_rate() .. "¢ now!")
            sound.play_wait("toll/nag_tone", Channel.BG01, { volume = NAG_TONE_VOLUME })
            sound.play("toll/nag_extend_call", Channel.BG01)
            sound.wait_min(Channel.BG01, 10.0)
        end
        task.wait(1)
        
    end
})

return agent