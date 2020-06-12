local S = SERVICE_MODULE("mark")
S:require_sound_bank('mark')
S:require_sound_bank('sewer')

S:set_idle_tick_during(PHONE_STATE_IDLE, PHONE_STATE_DIAL_TONE)
-- TODO: Implement "slow idle" mode for services
-- S:use_slow_idle()

local talked_during_call = false

-- args.path: path to this script file
function S.load(args)    
end

S:state(SERVICE_STATE_IDLE, {
    enter = function(self)
        while true do
            service.wait(120.0 + rand_float(0.0, 60.0))
            if chance(0.02) then
                service.start_call()
            end
        end
    end,
})

S:state(SERVICE_STATE_CALL_OUT, {
    enter = function(self)
        service.wait(rand_float(15.0, 30.0))
        service.end_call()
    end
})

S:state(SERVICE_STATE_CALL, {
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
            service.wait(rand_float(4.0, 7.0))
            sound.play("$mark/*", CHAN_PHONE1, { volume = 0.9 })
            self.talked_during_call = true
            if i == 1 or chance(1) then                
                S:send('denise', i == 1 and 'mark_start' or 'mark_talk')
            end
        end

        -- Pause briefly and hang up
        service.wait(rand_float(4.0, 8.0))
        service.end_call()
    end,
    exit = function(self)
        if self.talked_during_call == true then
            S:send('denise', 'mark_end')
        end
    end
})

return S