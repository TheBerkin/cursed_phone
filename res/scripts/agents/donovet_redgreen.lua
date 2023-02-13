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

local MONSTER_STATE_IDLE = 0
local MONSTER_STATE_MENACE = 1
local MONSTER_STATE_ATTACK = 2

--- @class RedGreenGame
local game = {
    --- Gameplay controls locked?
    controls_locked = false,
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
    monster = {
        state = MONSTER_STATE_IDLE,
        state_time = time_since()
    },
    --- @param self RedGreenGame
    reset = function(self)
        self.heartbeat_enabled = true
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
local function task_ambience()
    sound.play("$redgreen/ambient/amb_dungeon", Channel.PHONE08, { looping = true, skip = rand_float(0, 30), volume = 0.1 })

    agent.multi_task(
    function()
        local first_moment = true

        while true do
            agent.wait(first_moment and rand_float(0, 25) or rand_float(5, 30))
            sound.play("$redgreen/ambient/moment_drip_*", Channel.PHONE07, { volume = rand_float(0.01, 0.1), speed = rand_float(0.85, 1.1), interrupt = false })
            first_moment = false
        end
    end,
    function()
        while true do 
            agent.wait(rand_float(5, 20))
            if chance(0.35) then
                sound.play_wait("$redgreen/ambient/moment_rare_*", Channel.PHONE06, { volume = rand_float(0.005, 0.125), speed = rand_float(0.9, 1.1) })
            end
        end
    end)
end

local function task_monster_sounds()
    sound.play("ambient/static", Channel.PHONE04, { looping = true, volume = 0.175 })
    while true do 
        agent.wait(rand_float(1, 5))
        sound.play_wait("$redgreen/monster/croak_*", Channel.PHONE05, {
            volume = 0.3,
            fadein = chance(0.5) and 1 or 0,
            speed = rand_float(0.8, 1.25),
            skip = chance(0.4) and rand_float(0, 0.5) or 0 
        })
    end
end

--- @async
local function task_update_heartbeat()
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
local function task_update_heart_rate()
    while true do
        game.heart_rate = math.sin(engine_time() * TAU * 0.1) * 30 + 95
        agent.yield()
    end
end

--- @async
local function task_update_controls()
    while true do
        if not game.controls_locked then
            local digit = tonumber(agent.read_digit())
            if digit then
                if digit == 1 then
                    -- go
                    game.walking = true
                    module:log("Victim: Moving.")
                elseif game.walking then
                    -- stop
                    if not game.stop_digits_used[digit] then
                        -- allow stop
                        game.stop_digits_used[digit] = true
                        game.walking = false
                        module:log("Victim: Stopped.")
                    else
                        -- stop digit already used!
                        module:log("Victim: Stop already used!")
                    end
                end
            end
        end
    end
end

--- @async
local function task_update_footsteps()
    local function is_victim_stationary() return not game.walking end

    while true do
        if game.walking then
            agent.wait_cancel(rand_float(0.25, 0.6), is_victim_stationary)
            while game.walking do 
                sound.play("$redgreen/footstep_*", Channel.PHONE03, { volume = rand_float(0.2, 0.3), speed = rand_float(0.9, 1.1), interrupt = true })
                agent.wait_cancel(rand_float(0.8, 0.9), is_victim_stationary)
            end
            agent.wait(rand_float(0.2, 0.35))
            sound.play("$redgreen/footstep_*", Channel.PHONE03, { volume = rand_float(0.1, 0.2), speed = rand_float(0.8, 1), interrupt = true })
            if chance(0.5) then 
                agent.wait(rand_float(0.2, 0.4))
                sound.play("$redgreen/footstep_*", Channel.PHONE03, { volume = rand_float(0.1, 0.15), speed = rand_float(0.6, 0.8), interrupt = true })
            end
        end
        agent.yield()
    end
end

local function task_intro()

end

module:state(AgentState.CALL, {
    enter = function(self)
        agent.multi_task(
            task_ambience,
            -- task_intro,
            task_update_heartbeat,
            task_update_heart_rate,
            task_update_controls,
            task_update_footsteps,
            task_monster_sounds
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