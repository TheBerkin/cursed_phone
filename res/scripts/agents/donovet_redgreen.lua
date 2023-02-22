--[[ 

Donovet's Challenge: Red Light / Green Light

SOUND CHANNEL LAYOUT: 
* PHONE01:  VO (Donovet)
* PHONE02:  VO (Victim)
* PHONE03:  Victim Footsteps
* PHONE04:  Monster Proximity SFX
* PHONE05:  Monster Voice A
* PHONE06:  VO (Computer)
* PHONE07:  Victim Heartbeat
* Phone08:  Heart Monitor
* Phone09:  --
* Phone10:  Monster Voice B
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

local SOUNDSCAPE_VOLUME = 0.9

local VO_COMPUTER_DISTANCE_LINES = {
    [10] = "$redgreen/vo/computer_10",
    [20] = "$redgreen/vo/computer_20",
    [30] = "$redgreen/vo/computer_30",
    [40] = "$redgreen/vo/computer_40",
    [50] = "$redgreen/vo/computer_50",
    [60] = "$redgreen/vo/computer_60",
    [70] = "$redgreen/vo/computer_70",
    [80] = "$redgreen/vo/computer_80",
    [90] = "$redgreen/vo/computer_90",
    [100] = "$redgreen/vo/computer_100",
    [110] = "$redgreen/vo/computer_110",
    [120] = "$redgreen/vo/computer_120",
}

local HEART_RATE_BASE = 80
local HEART_RATE_MIN = 20
local HEART_RATE_MAX = 150
local HEART_RATE_LETHAL_MIN = 145
local HEART_RATE_STRESS_FACTOR = 2.8
local HEART_RATE_NOISE_FACTOR = 18.0
local HEART_MONITOR_VOLUME = 0.015
local HEARTBEAT_B_THRESHOLD = 65
local HEARTBEAT_C_THRESHOLD = 110
local HEARTBEAT_WIDTH = 0.03

local VICTIM_ESCAPE_DISTANCE = 120
local VICTIM_SPEED = 1.0
local VICTIM_FOOTSTEP_VOLUME = 0.8
local VICTIM_STATIONARY_STRESS_RATE = 0.08
local VICTIM_WALK_TEMP_STRESS = 2.0
local VICTIM_STOP_TEMP_STRESS_MIN = 2.5
local VICTIM_STOP_TEMP_STRESS_MAX = 3.75
local VICTIM_STOP_STRESS_MIN = 1.25
local VICTIM_STOP_STRESS_MAX = 2.75
local VICTIM_TEMP_STRESS_MAX = 10.0
local VICTIM_TEMP_STRESS_DECAY_RATE = 0.45

local VO_COMPUTER_VOLUME = 0.075
local VO_DONOVET_VOLUME = 0.125

local MONSTER_STATE_IDLE = 'idle'
local MONSTER_STATE_MENACE = 'menace'
local MONSTER_STATE_ATTACK = 'attack'

local MONSTER_IDLE_MIN_TIME = 6.0
local MONSTER_IDLE_MAX_TIME = 30.0
local MONSTER_IDLE_INTERVAL = 3.63
local MONSTER_IDLE_INTERVAL_P = 0.18
local MONSTER_IDLE_INTERVAL_TIMEOUT = MONSTER_IDLE_MAX_TIME - MONSTER_IDLE_MIN_TIME

local MONSTER_MENACE_DELAY = 3.75
local MONSTER_MENACE_MIN_TIME = 5.6
local MONSTER_MENACE_MAX_TIME = 13.0
local MONSTER_MENACE_STATIC_VOLUME = 0.15
local MONSTER_MENACE_STATIC_VOLUME_NOISE_SCALE = 0.4
local MONSTER_MENACE_STATIC_FADEIN_RATE = 0.4
local MONSTER_MENACE_STATIC_FADEOUT_RATE = 0.2
local MONSTER_MENACE_VOCAL_VOLUME = 0.4
local MONSTER_MENACE_VOCAL_FADEIN_RATE = 0.15
local MONSTER_MENACE_VOCAL_FADEOUT_RATE = 0.4

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
        heart_rate = HEART_RATE_BASE,
        --- Victim's current stress level (persistent)
        stress = 0.0,
        --- Victim's temporary stress level (decaying)
        temp_stress = 0.0,
        --- Victim's distance from the exit
        goal_distance = VICTIM_ESCAPE_DISTANCE,
        --- Last distance that the computer VO reported
        last_reported_distance = VICTIM_ESCAPE_DISTANCE,
        --- Is heart monitor running?
        ekg_enabled = false,
        --- Is the heart monitor FREAKING OUT?
        ekg_panic_mode = false,
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

function game.victim:decay_temp_stress(delta_time)
    self.temp_stress = math.clamp(self.temp_stress - VICTIM_TEMP_STRESS_DECAY_RATE * delta_time, 0.0, VICTIM_TEMP_STRESS_MAX)
end

function game.victim:add_temp_stress(amount)
    self.temp_stress = math.clamp(self.temp_stress + amount, 0.0, VICTIM_TEMP_STRESS_MAX)
end

function game.victim:add_stress(amount)
    self.stress = math.max(self.stress + amount, 0.0)
end

--- @return number?
function game.victim:get_last_reportable_distance()
    local min_rd = nil
    for k, _ in pairs(VO_COMPUTER_DISTANCE_LINES) do
        if self.goal_distance < k and (not min_rd or k < min_rd) then
            min_rd = k
        end
    end
    return min_rd
end

--- @return boolean
function game.victim:update_distance_report()
    local last_reportable_distance = self:get_last_reportable_distance()
    if last_reportable_distance ~= self.last_reported_distance then
        self.last_reported_distance = last_reportable_distance
        return true
    end
    return false
end

local function check_victim_detectable()
    return game.victim.walking or game.victim.heart_rate >= HEART_RATE_LETHAL_MIN
end

--- @type FsmStateTable
local MONSTER_STATES = {
    [MONSTER_STATE_IDLE] = function(self, from_state)
        game.monster.vocals_enabled = false
        agent.wait(MONSTER_IDLE_MIN_TIME)
        agent.chance_interval(MONSTER_IDLE_INTERVAL, MONSTER_IDLE_INTERVAL_P, MONSTER_IDLE_INTERVAL_TIMEOUT)
        self:transition(MONSTER_STATE_MENACE)
    end,
    [MONSTER_STATE_MENACE] = function(self)
        game.monster.vocals_enabled = true
        agent.wait(MONSTER_MENACE_DELAY)
        local menace_time = rand_float(MONSTER_MENACE_MIN_TIME, MONSTER_MENACE_MAX_TIME)
        if agent.wait_cancel(menace_time, check_victim_detectable) then
            self:transition(MONSTER_STATE_ATTACK)
        end
        self:transition(MONSTER_STATE_IDLE)
    end,
    [MONSTER_STATE_ATTACK] = function(self)
        game.controls_locked = true
        game.victim.walking = false
        game.monster.vocals_enabled = false
        sound.wait(Channel.PHONE05)
        sound.play("$redgreen/monster/scream", Channel.PHONE10, { volume = 0.35 })
        agent.wait(1.25)
        game.victim.ekg_panic_mode = true
        sound.wait(Channel.PHONE10)
        agent.end_call()
    end
}

local function on_monster_state_change(fsm, from, to)
    module:log("Monster state: " .. from .. " -> " .. to)
end

function game:reset()
    self.controls_locked = true
    table.clear(self.stop_digits_used)
    -- reset victim
    self.victim.heart_rate = HEART_RATE_BASE
    self.victim.stress = 0.0
    self.victim.temp_stress = 0.0
    self.victim.last_reported_distance = VICTIM_ESCAPE_DISTANCE
    self.victim.goal_distance = VICTIM_ESCAPE_DISTANCE
    self.victim.ekg_enabled = false
    self.victim.ekg_panic_mode = false
    self.victim.walking = false
    -- reset monster
    local monster_ai = Fsm.new(MONSTER_STATES, MONSTER_STATE_IDLE)
    monster_ai:on_transition(on_monster_state_change)
    self.monster.active = false
    self.monster.ai = monster_ai
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
    sound.set_channel_volume(Channel.BG01, SOUNDSCAPE_VOLUME)
    sound.set_channel_volume(Channel.BG02, SOUNDSCAPE_VOLUME)
    sound.set_channel_volume(Channel.BG03, SOUNDSCAPE_VOLUME)
    sound.set_channel_volume(Channel.BG04, SOUNDSCAPE_VOLUME)

    sound.play("$redgreen/ambient/amb_dungeon", Channel.BG01, { looping = true, skip = rand_float(0, 30), volume = 0.15 })

    agent.multi_task(
    function()
        local first_moment = true

        while true do
            agent.wait(first_moment and rand_float(0, 25) or rand_float(5, 30))
            sound.play("$redgreen/ambient/moment_drip_*", Channel.BG02, { volume = rand_float(0.01, 0.1), speed = rand_float(0.8, 1.15), interrupt = false })
            first_moment = false
        end
    end,
    function()
        while true do
            agent.wait(rand_float(5, 18))
            if chance(0.35) then
                sound.play_wait("$redgreen/ambient/moment_rare_*", Channel.BG03, { volume = rand_float(0.005, 0.125), speed = rand_float(0.9, 1.15) })
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
            sound.play("$redgreen/monster/proximity", Channel.PHONE04, { looping = true, volume = MONSTER_MENACE_STATIC_VOLUME })
            while true do
                agent.wait(rand_float(1, 5))
                if game.monster.vocals_enabled then
                    sound.play_wait("$redgreen/monster/croak_*", Channel.PHONE05, {
                        volume = rand_float(0.69, 1) * MONSTER_MENACE_VOCAL_VOLUME,
                        fadein = chance(0.5) and 1 or 0,
                        speed = rand_float(0.8, 1.25),
                        skip = chance(0.4) and rand_float(0, 0.5) or 0
                    })
                end
            end
        end,
        -- Volume control
        function()
            local proximity_noise_seed = rand_seed_32()
            local prev_time = engine_time()
            local static_volume = 0.0
            local vocal_volume = 0.0
            while true do
                local current_time = engine_time()
                local dt = current_time - prev_time
                local static_volume_noise = (perlin_sample(current_time, 0, 3, 12, 0.9, 2.0, proximity_noise_seed) + 1) * 0.5
                local static_volume_scale = 1.0 - static_volume_noise * MONSTER_MENACE_STATIC_VOLUME_NOISE_SCALE

                if game.monster.vocals_enabled then
                    static_volume = math.step_to(static_volume, 1.0, dt * MONSTER_MENACE_STATIC_FADEIN_RATE)
                    vocal_volume = math.step_to(vocal_volume, 1.0, dt * MONSTER_MENACE_VOCAL_FADEIN_RATE)
                else
                    static_volume = math.step_to(static_volume, 0.0, dt * MONSTER_MENACE_STATIC_FADEOUT_RATE)
                    vocal_volume = math.step_to(vocal_volume, 0.0, dt * MONSTER_MENACE_VOCAL_FADEOUT_RATE)
                end

                sound.set_channel_volume(Channel.PHONE04, (static_volume * static_volume_scale) ^ 2)
                sound.set_channel_volume(Channel.PHONE05, vocal_volume ^ 2)

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
            if victim.ekg_panic_mode then 
                gpio.write_pin(OUT_VIBRATE, GPIO_HIGH)
                agent.yield()
            else 
                gpio.write_pin(OUT_VIBRATE, GPIO_HIGH)
                sound.play(select_heartbeat_bank(victim.heart_rate), Channel.PHONE07, { speed = rand_float(0.9, 1.1) })
                sound.play("$redgreen/heart_monitor_beep", Channel.PHONE08, { volume = HEART_MONITOR_VOLUME })
                agent.wait(HEARTBEAT_WIDTH)
                gpio.write_pin(OUT_VIBRATE, GPIO_LOW)
                if victim.ekg_enabled then
                    agent.wait_dynamic(get_post_heartbeat_wait_time)
                end
            end
        end
    end
end

--- @async
local function task_update_heart_rate()
    local victim = game.victim
    local noise_seed = rand_seed_32()
    local is_critical = false

    while true do
        local noise = perlin_sample(engine_time(), 0, 3, 1, 0.5, 2.0, noise_seed)
        local bpm_factor_stress = (victim.stress + victim.temp_stress) * HEART_RATE_STRESS_FACTOR
        local bpm_factor_noise = noise * HEART_RATE_NOISE_FACTOR
        victim.heart_rate = math.clamp(HEART_RATE_BASE + bpm_factor_stress + bpm_factor_noise, HEART_RATE_MIN, HEART_RATE_MAX)
        if not is_critical and victim.heart_rate >= HEART_RATE_LETHAL_MIN then
            is_critical = true
            module:log("Victim heart rate is critical!")
        end
        agent.yield()
    end
end

--- @async
local function task_update_controls()
    local victim = game.victim

    while true do
        if not game.controls_locked then
            local digit = tonumber(agent.read_digit())
            if digit then
                if digit == 1 then
                    -- go
                    victim.walking = true
                    victim:add_temp_stress(VICTIM_WALK_TEMP_STRESS)
                    module:log("Victim: Moving.")
                elseif victim.walking then
                    -- stop
                    if not game.stop_digits_used[digit] then
                        -- allow stop
                        game.stop_digits_used[digit] = true
                        victim.walking = false
                        victim:add_stress(rand_float(VICTIM_STOP_STRESS_MIN, VICTIM_STOP_STRESS_MAX))
                        victim:add_temp_stress(rand_float(VICTIM_STOP_TEMP_STRESS_MIN, VICTIM_STOP_TEMP_STRESS_MAX))
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
                agent.wait_cancel(rand_float(0.675, 0.8), is_victim_stationary)
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

local function task_scenario_win()
    local victim = game.victim
    game.monster.active = false
    game.controls_locked = true
    victim.walking = false
    victim:add_temp_stress(5.0)
    sound.play_wait("$redgreen/escape_door", Channel.PHONE09, { volume = 0.35 })
    agent.wait(2.5)
    agent.end_call()
end

local function task_update_victim()
    local victim = game.victim
    local last_tick_time = engine_time()
    while true do
        if victim.goal_distance <= 0.0 and game.monster.ai:state() == MONSTER_STATE_IDLE then
            task_scenario_win()
        end

        local time = engine_time()
        local dt = time - last_tick_time
        last_tick_time = time

        victim:decay_temp_stress(dt)

        if victim.walking then
            local distance_delta = VICTIM_SPEED * dt
            local distance_prev = victim.goal_distance
            local distance_updated = math.max(0, distance_prev - distance_delta)
            victim.goal_distance = distance_updated

            -- Quick and dirty log of victim distance
            if math.abs(math.ceil(distance_prev) - math.ceil(distance_updated)) >= 1 then
                if victim:update_distance_report() then
                    local report_sound_path = VO_COMPUTER_DISTANCE_LINES[victim.last_reported_distance]
                    if report_sound_path then
                        sound.play(report_sound_path, Channel.PHONE06, { volume = VO_COMPUTER_VOLUME, interrupt = false })
                    end
                end
                module:log("Victim: " .. math.ceil(distance_updated) .. "m from exit.")
            end
        else
            victim:add_stress(dt * VICTIM_STATIONARY_STRESS_RATE)
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