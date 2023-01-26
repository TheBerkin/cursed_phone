local module = AGENT_MODULE("hangman", "7308432")

module:require_sound_bank("hangman")

module:state(AGENT_STATE_CALL_IN, {
    enter = function(self) 
        agent.wait(rand_float(4.0, 8.0))
        agent.accept_call()
    end
})

module:state(AGENT_STATE_CALL_OUT, {
    enter = function(self)
        agent.wait(rand_float(10.0, 20.0))
        agent.end_call()
    end
})

module:state(AGENT_STATE_CALL, {
    enter = function(self)
        print("Hangman: call started")
    end,
    tick = function(self)
        agent.wait(rand_float(0.5, 1.5))

        sound.play("$hangman/vo/donovet_intro_01", CHAN_PHONE1)
        agent.wait(rand_float(0.2, 0.9))

        -- Crank up chainsaw
        for i = 1, rand_int(2, 4) do
            sound.play_wait("$hangman/chainsaw_crank_*", CHAN_PHONE2)
            agent.wait(rand_float(0.5, 1.5))
        end

        -- Start chainsaw
        sound.play("$hangman/chainsaw_start", CHAN_PHONE2)
        sound.play("$hangman/chainsaw_idle", CHAN_PHONE2, { interrupt = false, looping = true })
        agent.wait(rand_float(2.0, 3.0))

        -- Rev it up!
        while true do
            agent.wait(rand_float(1.5, 5.0))
            sound.set_channel_volume(CHAN_PHONE2, 0.5)
            sound.play("$hangman/chainsaw_rev_*", CHAN_PHONE3, { speed = rand_float(0.9, 1.1) })
            sound.wait_max(CHAN_PHONE3, 0.8)
            sound.set_channel_volume(CHAN_PHONE2, 1)
        end
    end,
    exit = function(self)
        print("Hangman: ending call")
    end
})

return module