local module = create_agent("denise")

local IN_MOTION = 16
local OUT_VIBRATE = 27

module:set_idle_tick_during(PHONE_STATE_IDLE, PHONE_STATE_DIAL_TONE)
-- S:suspend()

local msg_handlers = {
    ["mark_start"] = function(self, msg_data)
        -- Mark call starts
        agent.wait(rand_float(0.8, 2.0))
        if not sound.is_busy(CHAN_SOUL1) then
            sound.play("denise/reactions/mark_greet/*", CHAN_SOUL1, { interrupt = false })
        end
    end,
    ["mark_talk"] = function(self, msg_data)
        -- Mark says something
        agent.wait(rand_float(1.75, 2.5))
        if not sound.is_busy(CHAN_SOUL1) then
            sound.play("denise/reactions/mark_plead/*", CHAN_SOUL1, { interrupt = false })
        end
    end,
    ["mark_end"] = function(self, msg_data)
        -- Mark call ends
        agent.wait(rand_float(1.0, 3.0))
        if not sound.is_busy(CHAN_SOUL1) then
            sound.play("denise/reactions/mark_post/*", CHAN_SOUL1, { interrupt = false })
        end
    end
}

module:on_load(function()
    gpio.register_input(IN_MOTION, GPIO_PULL_DOWN, 0.25)
    gpio.register_output(OUT_VIBRATE)
    gpio.write_pin(OUT_VIBRATE, GPIO_LOW)
end)

local function vibrate_excitedly()
    for i = 1, rand_int(1, 4) do
        local vibe_period = rand_float(0.025, 0.075)
        local vibe_pw = rand_int(0, 3) == 0 and rand_float(vibe_period * 0.25, vibe_period) or vibe_period
        gpio.set_pwm(OUT_VIBRATE, vibe_period, vibe_pw)
        agent.wait(rand_float(0.1, 0.5))
    end
    gpio.clear_pwm(OUT_VIBRATE)
end

module:state(AGENT_STATE_IDLE, {
    enter = function(self)
        agent.wait_until(function() return gpio.read_pin(IN_MOTION) == GPIO_LOW end)
    end,

    tick = function(self)
        agent.wait_until(function() return gpio.read_pin(IN_MOTION) == GPIO_HIGH end)
        if chance(0.1) then
            agent.wait(rand_float(0.5, 1))
            agent.start_call()
            -- vibrate_excitedly()
        end        
        agent.wait(rand_float(2, 4))
        agent.wait_until(function() return gpio.read_pin(IN_MOTION) == GPIO_LOW end)
    end,

    message = function(self, sender, msg_type, msg_data)
        local handler = msg_handlers[msg_type]
        if type(handler) == 'function' then
            handler(self, msg_data)
        end
    end,

    exit = function(self)

    end
})

module:state(AGENT_STATE_CALL_OUT, {
    enter = function(self)
        agent.wait(rand_float(10, 15))
        agent.end_call()
    end,
})

module:state(AGENT_STATE_CALL, {
    enter = function(self)
        agent.wait(rand_float(2, 3))
        for i = 1, rand_int(1, 3) do
            sound.play_wait("denise/seeing/*", CHAN_PHONE1, { interrupt = true })
        agent.wait(rand_float(1, 3))
        end
        agent.wait(rand_float(1, 2))
        agent.end_call()
    end,
})

module:on_unload(function(self)
    gpio.unregister(IN_MOTION)
    gpio.unregister(OUT_VIBRATE)
end)

return module