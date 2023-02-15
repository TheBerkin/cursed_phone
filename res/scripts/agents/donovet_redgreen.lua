--[[ 

Donovet's Challenge: Red Light / Green Light

SOUND CHANNEL LAYOUT: 
* PHONE01:  VO (Donovet)
* PHONE02:  VO (Victim)
* PHONE03:  Victim Footsteps
* PHONE04:  Monster Proximity SFX
* PHONE05:  Monster Voice
* PHONE06:  Heart Monitor
* PHONE07:  --
* Phone08:  --
* BG01:     Soundscape (Loop)
* BG02:     Soundscape (Moments - Dripping)
* BG03:     Soundscape (Moments - Rare)
* BG04:     --
]]

local module = create_agent("donovet_redgreen", "6661")
module:require_sound_bank("redgreen")
module:accept_all_calls() -- TODO: Remove this later!

-- constants
local TAU = math.pi * 2.0
local HEARTBEAT_B_THRESHOLD = 65
local HEARTBEAT_C_THRESHOLD = 110
local OUT_VIBRATE = 27
local HEART_RATE_START = 65
local HEARTBEAT_WIDTH = 0.03
local VICTIM_ESCAPE_DISTANCE = 120
local VICTIM_SPEED = 1
local FOOTSTEP_VOLUME = 0.65

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
    --- Monster state
    monster = {
        --- State code of the current state
        state = MONSTER_STATE_IDLE,
        --- Active time of the current state
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
local function task_soundscape()
    sound.play("$redgreen/ambient/amb_dungeon", Channel.BG01, { looping = true, skip = rand_float(0, 30), volume = 0.15 })

    agent.multi_task(
    function()
        local first_moment = true

        while true do
            agent.wait(first_moment and rand_float(0, 25) or rand_float(5, 30))
            sound.play("$redgreen/ambient/moment_drip_*", Channel.BG02, { volume = rand_float(0.01, 0.1), speed = rand_float(0.85, 1.1), interrupt = false })
            first_moment = false
        end
    end,
    function()
        while true do 
            agent.wait(rand_float(5, 20))
            if chance(0.35) then
                sound.play_wait("$redgreen/ambient/moment_rare_*", Channel.BG03, { volume = rand_float(0.005, 0.125), speed = rand_float(0.9, 1.1) })
            end
        end
    end)
end

--- @async
local function task_monster_sounds()
    sound.play("ambient/static", Channel.PHONE04, { looping = true, volume = 0.1 })
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
local function task_heartbeat_sounds()
    agent.wait(1.0)
    while true do
        agent.wait_until(function() return game.heartbeat_enabled end)
        sound.play_wait("$redgreen/vo/computer_ekg_ready", Channel.PHONE06, { volume = 0.2 })
        while game.heartbeat_enabled do
            gpio.write_pin(OUT_VIBRATE, GPIO_HIGH)
            sound.play("$redgreen/heart_monitor_beep", Channel.PHONE07, { volume = 0.05 })
            sound.play(select_heartbeat_bank(game.heart_rate), Channel.PHONE06, { speed = rand_float(0.9, 1.1) })
            agent.wait(HEARTBEAT_WIDTH)
            gpio.write_pin(OUT_VIBRATE, GPIO_LOW)
            if game.heartbeat_enabled then
                agent.wait_dynamic(get_post_heartbeat_wait_time)
            end
        end
    end
end

--- @async
local function task_update_heart_rate()
    local noise_seed = rand_seed_32()

    while true do
        local noise = (perlin_sample(engine_time(), 0, 3, 1, 0.5, 2.0, noise_seed) + 1) * 0.5
        game.heart_rate = noise * 30 + 95
        -- module:log("|" .. string.rep("=", noise * 36) .. string.rep(" ", (1 - noise) * 36) .. "| " .. game.heart_rate)
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
local function task_footstep_sounds()
    local function is_victim_stationary() return not game.walking end

    while true do
        if game.walking then
            agent.wait_cancel(rand_float(0.25, 0.6), is_victim_stationary)
            while game.walking do 
                sound.play("$redgreen/footstep_*", Channel.PHONE03, { volume = rand_float(0.4, 0.6) * FOOTSTEP_VOLUME, speed = rand_float(0.9, 1.1), interrupt = true })
                agent.wait_cancel(rand_float(0.8, 0.9), is_victim_stationary)
            end
            agent.wait(rand_float(0.2, 0.35))
            sound.play("$redgreen/footstep_*", Channel.PHONE03, { volume = rand_float(0.2, 0.4) * FOOTSTEP_VOLUME, speed = rand_float(0.8, 1), interrupt = true })
            if chance(0.5) then 
                agent.wait(rand_float(0.2, 0.4))
                sound.play("$redgreen/footstep_*", Channel.PHONE03, { volume = rand_float(0.2, 0.5) * FOOTSTEP_VOLUME, speed = rand_float(0.6, 0.8), interrupt = true })
            end
        end
        agent.yield()
    end
end

local function task_update_victim()

end

local function task_intro()

end

module:state(AgentState.CALL, {
    enter = function(self)
        agent.multi_task(
            task_soundscape,
            -- task_intro,
            task_heartbeat_sounds,
            task_update_heart_rate,
            task_update_controls,
            task_footstep_sounds,
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