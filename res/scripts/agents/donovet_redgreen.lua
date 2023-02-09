local module = create_agent("donovet_redgreen", "6661")
module:require_sound_bank("redgreen")
module:accept_all_calls() -- TODO: Remove this later!

local HEARTBEAT_B_THRESHOLD = 65
local HEARTBEAT_C_THRESHOLD = 110
local TAU = math.pi * 2.0
local OUT_VIBRATE = 27
local HEARTBEAT_WIDTH = 0.1

local heartbeat_enabled = true
local heart_rate = 65

module:state(AgentState.IDLE, { 
    tick = function(self)
        agent.wait(rand_float(10, 20))
        if chance(0.25) then 
            -- agent.start_call()
        end
    end
})

module:on_load(function(self)
    gpio.register_output(OUT_VIBRATE)
end)


module:state(AgentState.CALL_OUT, {
    enter = function(self)
        agent.wait(rand_float(20, 30))
        agent.end_call()
    end
})

--- @return number
local function get_post_heartbeat_wait_time()
    return math.max(0, 60.0 / heart_rate - HEARTBEAT_WIDTH)
end

local function select_heartbeat_bank(bpm) 
    if bpm >= HEARTBEAT_C_THRESHOLD then
        return "$redgreen/heartbeat_c_*"
    elseif bpm >= HEARTBEAT_B_THRESHOLD then
        return "$redgreen/heartbeat_b_*"
    else
        return "$redgreen/heartbeat_a_*"
    end
end

--- @async
local function do_heartbeat()
    while true do
        if heartbeat_enabled then
            gpio.write_pin(OUT_VIBRATE, GPIO_HIGH)
            sound.play(select_heartbeat_bank(heart_rate), Channel.PHONE01, { speed = rand_float(0.9, 1.1) })
            agent.wait(HEARTBEAT_WIDTH)
            gpio.write_pin(OUT_VIBRATE, GPIO_LOW)
            if heartbeat_enabled then
                agent.wait_dynamic(get_post_heartbeat_wait_time)
            end
        else
            agent.yield()
        end
    end
end

--- @async
local function update_heart_rate()
    while true do
        heart_rate = math.sin(engine_time() * TAU * 0.1) * 30 + 95
        agent.yield()
    end
end

module:state(AgentState.CALL, {
    enter = function(self)
        agent.multi_task(do_heartbeat, update_heart_rate)
    end,

    exit = function(self)
        gpio.clear_pwm(OUT_VIBRATE)
        gpio.write_pin(OUT_VIBRATE, GPIO_LOW)
    end
})

module:on_unload(function(self)
    gpio.unregister(OUT_VIBRATE)
end)

return module