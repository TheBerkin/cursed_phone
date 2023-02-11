local module = create_agent("donovet_redgreen", "6661")
module:require_sound_bank("redgreen")
module:accept_all_calls() -- TODO: Remove this later!

-- constants
local HEARTBEAT_B_THRESHOLD = 65
local HEARTBEAT_C_THRESHOLD = 110
local TAU = math.pi * 2.0
local OUT_VIBRATE = 27
local HEART_RATE_START = 65
local HEARTBEAT_WIDTH = 0.03
local VICTIM_ESCAPE_DISTANCE = 30


--- @class RedGreenGame
local game = {
    --- Is victim's heartbeat audible?
    heartbeat_enabled = true,
    --- Victim's current heart rate
    heart_rate = HEART_RATE_START,
    --- Stop digits already used by player
    stop_digits_used = {},
    --- Victim's distance from the exit
    victim_distance = VICTIM_ESCAPE_DISTANCE,
    --- Is victim currently walking?
    walking = false,
    --- @param self RedGreenGame
    reset = function(self)
        self.heartbeat_enabled = false
        self.heart_rate = HEART_RATE_START
        table.clear(self.stop_digits_used)
        self.victim_distance = VICTIM_ESCAPE_DISTANCE
        self.walking = false
    end
}

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
    return math.max(0, 60.0 / game.heart_rate - HEARTBEAT_WIDTH)
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
        if game.heartbeat_enabled then
            gpio.write_pin(OUT_VIBRATE, GPIO_HIGH)
            sound.play(select_heartbeat_bank(game.heart_rate), Channel.PHONE01, { speed = rand_float(0.9, 1.1) })
            agent.wait(HEARTBEAT_WIDTH)
            gpio.write_pin(OUT_VIBRATE, GPIO_LOW)
            if game.heartbeat_enabled then
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
        game.heart_rate = math.sin(engine_time() * TAU * 0.1) * 30 + 95
        agent.yield()
    end
end

local function update_movement_controls()
    while true do
        local digit = tonumber(agent.read_digit())
        if digit then
            if digit == 1 then
                -- go
                game.walking = true
                module:log("Victim: Moving.")
            else
                -- stop
                if not game.stop_digits_used[digit] then
                    -- allow stop
                    game.stop_digits_used[digit] = true
                    game.walking = false
                    module:log("Victim: Stopped.")
                else
                    -- stop digit already used!
                    module:log("Victim: Can't stop.")
                end
            end
        end
    end
end

local function update_footsteps()
    local function is_victim_stationary() return not game.walking end

    while true do
        if game.walking then
            sound.play("$redgreen/footstep_*", Channel.PHONE03, { volume = rand_float(0.2, 0.3), speed = rand_float(0.9, 1.1), interrupt = true })
            agent.wait_cancel(rand_float(0.8, 0.9), is_victim_stationary)
        else
            agent.yield()
        end
    end
end

module:state(AgentState.CALL, {
    enter = function(self)
        agent.multi_task(
            do_heartbeat,
            update_heart_rate,
            update_movement_controls,
            update_footsteps
        )
    end,

    exit = function(self)
        gpio.clear_pwm(OUT_VIBRATE)
        gpio.write_pin(OUT_VIBRATE, GPIO_LOW)
        game:reset()
    end
})

module:on_unload(function(self)
    gpio.unregister(OUT_VIBRATE)
end)

return module