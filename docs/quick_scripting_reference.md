# Quick Scripting Reference

## Agent modules

Below is a skeleton for a basic agent module.

```lua
-- Module definition. Only the first argument, the name, is required.
local S = create_agent("agent_name", "1234567", AgentRole.NORMAL)

-- When not called, sound banks load only during CALL and CALL_OUT
S:set_sound_banks_loaded_during(AgentState.CALL, AgentState.CALL_IN, AgentState.CALL_OUT)

-- Called when agent has finished loading
S:on_load(function(self) end)

-- State machine definition
S:state(AGENT_STATE_IDLE, {
    -- Called when state is entered
    enter = function(self)
    end,

    -- Called when state is updated
    tick = function(self)
    end,

    -- Called when a message is received from another state
    message = function(self, sender, msg_type, msg_data)
    end,

    -- Called when state is about to exit
    exit = function(self)
    end
})

-- Called when shutting down engine
S:on_unload(function(self) end)

-- Make sure to return the completed module at the end
return S
```