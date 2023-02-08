local module = create_agent("hangman", "7308432")

module:require_sound_bank("hangman")

module:state(AgentState.CALL_IN, {
    enter = function(self) 
        agent.wait(rand_float(4.0, 8.0))
        agent.accept_call()
    end
})

module:state(AgentState.CALL_OUT, {
    enter = function(self)
        agent.wait(rand_float(10.0, 20.0))
        agent.end_call()
    end
})

module:state(AgentState.CALL, {
    enter = function(self)
        print("Hangman: call started")
    end,
    tick = function(self)
        agent.wait(rand_float(0.5, 1.5))

        sound.play("$hangman/vo/donovet_intro_01", Channel.PHONE01)
        agent.wait(rand_float(0.2, 0.9))

        -- Crank up chainsaw
        for i = 1, rand_int(2, 4) do
            sound.play_wait("$hangman/chainsaw_crank_*", Channel.PHONE02)
            agent.wait(rand_float(0.5, 1.5))
        end

        -- Start chainsaw
        sound.play("$hangman/chainsaw_start", Channel.PHONE02)
        sound.play("$hangman/chainsaw_idle", Channel.PHONE02, { interrupt = false, looping = true })
        agent.wait(rand_float(2.0, 3.0))

        -- Rev it up!
        while true do
            agent.wait(rand_float(1.5, 5.0))
            sound.set_channel_volume(Channel.PHONE02, 0.5)
            sound.play("$hangman/chainsaw_rev_*", Channel.PHONE03, { speed = rand_float(0.9, 1.1) })
            sound.wait_max(Channel.PHONE03, 0.8)
            sound.set_channel_volume(Channel.PHONE02, 1)
        end
    end,
    exit = function(self)
        print("Hangman: ending call")
    end
})

return module