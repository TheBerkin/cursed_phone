--[[ 

Donovet's Challenge: Red Light / Green Light

SOUND CHANNEL LAYOUT: 
* PHONE01:  VO (Donovet)
* PHONE02:  VO (Victim)
* PHONE03:  Victim Footsteps
* PHONE04:  Monster Proximity SFX
* PHONE05:  Monster Voice
* PHONE06:  VO (Computer) / Heart Monitor
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

local OUT_VIBRATE = 27

local HEART_RATE_START = 65
local HEART_MONITOR_VOLUME = 0.015
local HEARTBEAT_B_THRESHOLD = 65
local HEARTBEAT_C_THRESHOLD = 110
local HEARTBEAT_WIDTH = 0.03

local VICTIM_ESCAPE_DISTANCE = 120
local VICTIM_SPEED = 1
local VICTIM_FOOTSTEP_VOLUME = 1

local VO_COMPUTER_VOLUME = 0.075
local VO_DONOVET_VOLUME = 0.125

local MONSTER_STATE_IDLE = 0
local MONSTER_STATE_MENACE = 1
local MONSTER_STATE_ATTACK = 2

local MONSTER_IDLE_MIN_TIME = 5.0
local MONSTER_IDLE_MAX_TIME = 30.0
local MONSTER_IDLE_INTERVAL = 1.83
local MONSTER_IDLE_INTERVAL_P = 0.2
local MONSTER_IDLE_INTERVAL_TIMEOUT = MONSTER_IDLE_MAX_TIME - MONSTER_IDLE_MIN_TIME

local MONSTER_MENACE_DELAY = 2.75
local MONSTER_MENACE_MIN_TIME = 5.6
local MONSTER_MENACE_MAX_TIME = 13.0
local MONSTER_MENACE_STATIC_FADEIN_RATE = 1.6
local MONSTER_MENACE_VOCAL_FADEIN_RATE = 0.2
local MONSTER_MENACE_STATIC_FADEOUT_RATE = 0.5
local MONSTER_MENACE_VOCAL_FADEOUT_RATE = 1.4

--- @package
--- @class RedGreenGame
local game = {
    --- Gameplay controls locked?
    controls_locked = true,
    --- Stop digits already used by player
    stop_digits_used = {},
    --- @class RedGreenVictim
    victim = {
        --- Victim's current heart rate
        heart_rate = HEART_RATE_START,
        --- Victim's current stress level (persistent)
        stress = 0.0,
        --- Victim's temporary stress level (decaying)
        temp_stress = 0.0,
        --- Victim's distance from the exit
        goal_distance = VICTIM_ESCAPE_DISTANCE,
        --- Is heart monitor running?
        ekg_enabled = false,
        --- Is victim currently walking?
        walking = false,
    },
    --- @class RedGreenMonster
    monster = {
        --- Is the monster AI active?
        active = false,
        --- @type Fsm
        ai = nil,
        --- Is the monster vocalizing?
        vocals_enabled = false
    }
}

local function check_victim_detectable()
    return game.victim.walking
end

--- @type FsmStateTable
local MONSTER_STATES = {
    [MONSTER_STATE_IDLE] = function(self, from_state)
        module:log("Monster is idle.")
        game.monster.vocals_enabled = false
        if from_state == MONSTER_STATE_MENACE then
            game.victim.stress = game.victim.stress + rand_float(4, 8)
        end
        agent.wait(MONSTER_IDLE_MIN_TIME)
        agent.chance_interval(MONSTER_IDLE_INTERVAL, MONSTER_IDLE_INTERVAL_P, MONSTER_IDLE_INTERVAL_TIMEOUT)
        self:transition(MONSTER_STATE_MENACE)
    end,
    [MONSTER_STATE_MENACE] = function(self)
        module:log("Monster is menacing.")
        game.monster.vocals_enabled = true
        game.victim.stress = game.victim.stress + rand_float(4, 8)
        agent.wait(MONSTER_MENACE_DELAY)
        local menace_time = rand_float(MONSTER_MENACE_MIN_TIME, MONSTER_MENACE_MAX_TIME)
        if agent.wait_cancel(menace_time, check_victim_detectable) then
            self:transition(MONSTER_STATE_ATTACK)
        end
        self:transition(MONSTER_STATE_IDLE)
    end,
    [MONSTER_STATE_ATTACK] = function(self)
        module:log("Monster is attacking.")
        game.controls_locked = true
        game.victim.walking = false
        -- insert bloodcurdling screams
        agent.wait(3.0)
        agent.end_call()
    end
}

function game:reset()
    self.controls_locked = true
    table.clear(self.stop_digits_used)
    -- reset victim
    self.victim.heart_rate = HEART_RATE_START
    self.victim.stress = 0.0
    self.victim.temp_stress = 0.0
    self.victim.goal_distance = VICTIM_ESCAPE_DISTANCE
    self.victim.ekg_enabled = false
    self.victim.walking = false
    -- reset monster
    self.monster.active = false
    self.monster.ai = Fsm.new(MONSTER_STATES, MONSTER_STATE_IDLE)
end

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
    return math.max(0, 60.0 / game.victim.heart_rate - HEARTBEAT_WIDTH)
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
    sound.set_channel_volume(Channel.PHONE04, 0)
    sound.set_channel_volume(Channel.PHONE05, 0)

    agent.multi_task(
        -- Vocalizations
        function()
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
        end,
        -- Volume control
        function()
            local prev_time = engine_time()
            while true do
                local current_time = engine_time()
                local dt = current_time - prev_time

                local prev_static_volume = sound.get_channel_volume(Channel.PHONE04)
                local prev_vocal_volume = sound.get_channel_volume(Channel.PHONE05)
                local target_static_volume = prev_static_volume
                local target_vocal_volume = prev_vocal_volume

                if game.monster.vocals_enabled then
                    target_static_volume = math.step_to(prev_static_volume, 1, dt * MONSTER_MENACE_STATIC_FADEIN_RATE)
                    target_vocal_volume = math.step_to(prev_vocal_volume, 1, dt * MONSTER_MENACE_VOCAL_FADEIN_RATE)
                else
                    target_static_volume = math.step_to(prev_static_volume, 0, dt * MONSTER_MENACE_STATIC_FADEOUT_RATE)
                    target_vocal_volume = math.step_to(prev_vocal_volume, 0, dt * MONSTER_MENACE_VOCAL_FADEOUT_RATE)
                end

                sound.set_channel_volume(Channel.PHONE04, target_static_volume)
                sound.set_channel_volume(Channel.PHONE05, target_vocal_volume)

                prev_time = current_time
                agent.yield()
            end
        end
    )
end

--- @async
local function task_heartbeat_sounds()
    agent.wait(1.0)
    local victim = game.victim
    while true do
        agent.wait_until(function() return victim.ekg_enabled end)
        sound.play_wait("$redgreen/vo/computer_ekg_ready", Channel.PHONE06, { volume = VO_COMPUTER_VOLUME })
        while victim.ekg_enabled do
            gpio.write_pin(OUT_VIBRATE, GPIO_HIGH)
            sound.play("$redgreen/heart_monitor_beep", Channel.PHONE07, { volume = HEART_MONITOR_VOLUME })
            sound.play(select_heartbeat_bank(victim.heart_rate), Channel.PHONE06, { speed = rand_float(0.9, 1.1) })
            agent.wait(HEARTBEAT_WIDTH)
            gpio.write_pin(OUT_VIBRATE, GPIO_LOW)
            if victim.ekg_enabled then
                agent.wait_dynamic(get_post_heartbeat_wait_time)
            end
        end
    end
end

--- @async
local function task_update_heart_rate()
    local victim = game.victim
    local noise_seed = rand_seed_32()

    while true do
        local noise = perlin_sample(engine_time(), 0, 3, 1, 0.5, 2.0, noise_seed)
        local bpm_factor_stress = victim.stress
        victim.heart_rate = noise * 30 + 95 + bpm_factor_stress
        -- module:log("|" .. string.rep("=", noise * 36) .. string.rep(" ", (1 - noise) * 36) .. "| " .. game.heart_rate)
        agent.yield()
    end
end

--- @async
local function task_update_controls()
    local v = game.victim

    while true do
        if not game.controls_locked then
            local digit = tonumber(agent.read_digit())
            if digit then
                if digit == 1 then
                    -- go
                    v.walking = true
                    module:log("Victim: Moving.")
                elseif v.walking then
                    -- stop
                    if not game.stop_digits_used[digit] then
                        -- allow stop
                        game.stop_digits_used[digit] = true
                        v.walking = false
                        module:log("Victim: Stopped.")
                    else
                        -- stop digit already used!
                        module:log("Victim: Stop already used!")
                    end
                end
            end
        else
            agent.yield()
        end
    end
end

--- @async
local function task_footstep_sounds()
    local v = game.victim
    local function is_victim_stationary() return not v.walking end

    while true do
        if v.walking then
            agent.wait_cancel(rand_float(0.25, 0.6), is_victim_stationary)
            while v.walking do 
                sound.play("$redgreen/footstep_*", Channel.PHONE03, { volume = rand_float(0.4, 0.6) * VICTIM_FOOTSTEP_VOLUME, speed = rand_float(0.9, 1.1), interrupt = true })
                agent.wait_cancel(rand_float(0.8, 0.9), is_victim_stationary)
            end
            agent.wait(rand_float(0.2, 0.35))
            sound.play("$redgreen/footstep_*", Channel.PHONE03, { volume = rand_float(0.2, 0.4) * VICTIM_FOOTSTEP_VOLUME, speed = rand_float(0.8, 1), interrupt = true })
            if chance(0.5) then 
                agent.wait(rand_float(0.2, 0.4))
                sound.play("$redgreen/footstep_*", Channel.PHONE03, { volume = rand_float(0.2, 0.5) * VICTIM_FOOTSTEP_VOLUME, speed = rand_float(0.6, 0.8), interrupt = true })
            end
        end
        agent.yield()
    end
end

local function task_update_victim()
    local v = game.victim
    local last_tick_time = engine_time()
    while true do
        local time = engine_time()
        local dt = time - last_tick_time
        last_tick_time = time

        if v.walking then
            local distance_delta = VICTIM_SPEED * dt
            local distance_prev = v.goal_distance
            local distance_updated = math.max(0, distance_prev - distance_delta)
            v.goal_distance = distance_updated

            -- Quick and dirty log of victim distance
            if math.abs(math.ceil(distance_prev) - math.ceil(distance_updated)) >= 1 then
                module:log("Victim: " .. math.ceil(distance_updated) .. "m from exit.")
            end
        end
        agent.yield()
    end
end

local function task_update_monster_ai()
    local monster = game.monster

    while true do
        if monster.active then
            monster.ai:tick()
        else
            agent.yield()
        end
    end
end

local function task_intro()
    agent.wait(3)

    -- sound.play_wait("$redgreen/vo/intro/01_donovet", Channel.PHONE01, { volume = VO_DONOVET_VOLUME })
    -- agent.wait(1.2)
    -- sound.play_wait("$redgreen/vo/intro/02_donovet", Channel.PHONE01, { volume = VO_DONOVET_VOLUME })
    -- agent.wait(1.2)
    -- sound.play("$redgreen/vo/intro/03_donovet", Channel.PHONE01, { volume = VO_DONOVET_VOLUME })
    -- agent.wait(9.7)

    game.victim.ekg_enabled = true
    agent.wait(1.6)
    game.controls_locked = false
    game.monster.active = true
end

module:state(AgentState.CALL, {
    enter = function(self)
        game:reset()

        agent.multi_task(
            task_intro,
            task_soundscape,
            task_heartbeat_sounds,
            task_footstep_sounds,
            task_monster_sounds,
            task_update_heart_rate,
            task_update_controls,
            task_update_victim,
            task_update_monster_ai
        )
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