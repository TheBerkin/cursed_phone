local agent = AgentModule("operator", "0")
agent:set_custom_price(0)

agent:state(AgentState.CALL, {
    enter = function(self)
        -- TODO: Implement Operator
        while true do
            local digit = task.read_digit()
            log.info("Got digit '" .. digit .. "'")
        end
    end,
    exit = function(self)
    end
})


agent:state(AgentState.CALL_IN, {
    tick = function(self)
        task.wait(randf(1.0, 3.0))
        task.accept_call()
    end
})

return agent