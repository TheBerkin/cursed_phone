# Quick Scripting Reference

## Service modules

Below is a skeleton for a basic phone service module.

```lua
-- Module definition. Only the first argument, the name, is required.
local S = SERVICE_MODULE("service_name", "1234567", SERVICE_ROLE_NORMAL)

-- Special behavior functions
S:set_idle_tick_during(PHONE_STATE_IDLE, PHONE_STATE_DIAL_TONE)

-- Called when service is first loaded
function S.load(args)    
end

-- State machine definition
S:state(SERVICE_STATE_IDLE, {
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
function S.unload(args)
end

return S
```