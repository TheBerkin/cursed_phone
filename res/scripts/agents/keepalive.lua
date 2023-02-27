-- Terrible hacky workaround for the sound device randomly refusing playback after an arbitrary amount of time
local module = create_agent("keepalive")

module:state(AgentState.IDLE, {
    enter = function(self)
        while true do
            agent.wait(300)
            sound.play("silence", Channel.DEBUG)
        end
    end
})

return module