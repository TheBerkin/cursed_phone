local S = AGENT_MODULE("mark", "111")
S:require_sound_bank('mark')
S:require_sound_bank('sewer')

S:set_idle_tick_during(PHONE_STATE_IDLE, PHONE_STATE_DIAL_TONE)
-- TODO: Implement "slow idle" mode for agents
-- S:use_slow_idle()

local talked_during_call = false

-- args.path: path to this script file
function S.load(args)    
end

S:state(AGENT_STATE_IDLE, {
    enter = function(self)
        while true do
            agent.wait(120.0 + rand_float(0.0, 60.0))
            if chance(0.02) then
                agent.start_call()
            end
        end
    end,
})

S:state(AGENT_STATE_CALL_OUT, {
    enter = function(self)
        agent.wait(rand_float(15.0, 30.0))
        agent.end_call()
    end
})

S:state(AGENT_STATE_CALL, {
    enter = function(self)
        -- Start soundscape
        sound.play("$sewer/amb_drips", CHAN_PHONE2, { volume = 0.25, looping = true })
        sound.play("$sewer/amb_flies", CHAN_PHONE3, { volume = 0.05, looping = true })
        sound.play("$sewer/amb_hum", CHAN_PHONE4, { volume = 0.01, looping = true })
        sound.play("$sewer/amb_pain", CHAN_PHONE4, { volume = 0.1, looping = true })

        self.talked_during_call = false

        -- Say a few things
        for i = 1, rand_int(2, 4) do
            sound.wait(CHAN_PHONE1)
            agent.wait(rand_float(4.0, 7.0))
            sound.play("$mark/*", CHAN_PHONE1, { volume = 0.9 })
            self.talked_during_call = true
            if i == 1 or chance(0.7) then                
                S:send('denise', i == 1 and 'mark_start' or 'mark_talk')
            end
        end

        -- Pause briefly and hang up
        agent.wait(rand_float(4.0, 8.0))
        agent.end_call()
    end,
    exit = function(self)
        if self.talked_during_call == true then
            S:send('denise', 'mark_end')
        end
    end
})

return S