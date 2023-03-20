local module = new_agent("operator", "0")
module:set_custom_price(0)

module:state(AgentState.CALL, {
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


module:state(AgentState.CALL_IN, {
    tick = function(self)
        task.wait(randf(1.0, 3.0))
        task.accept_call()
    end
})

return module