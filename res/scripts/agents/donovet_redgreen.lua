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
* Phone09:  Escape Door
* Phone10:  Monster Voice B
* BG01:     Soundscape (Loop)
* BG02:     Soundscape (Moments - Dripping)
* BG03:     Soundscape (Moments - Rare + Screams)
* BG04:     Soundscape (Phantom Footsteps)
]]

local module = create_agent("donovet_redgreen", "6661")
module:require_sound_bank("redgreen")
module:accept_all_calls() -- TODO: Remove this later!

-- constants
local TAU = math.pi * 2.0

local OUT_VIBRATE = 27

local SOUNDSCAPE_VOLUME = 0.9

local PHANTOM_FOOTSTEP_VOLUME = 0.8
local PHANTOM_FOOTSTEP_PLAYBACK_SPEED = 0.8
local PHANTOM_FOOTSTEP_TEMP_STRESS_MIN = 6.0
local PHANTOM_FOOTSTEP_TEMP_STRESS_MAX = 12.0
local PHANTOM_FOOTSTEP_SENSITIVITY = 0.75

local HEART_RATE_BASE = 75
local HEART_RATE_MIN = 20
local HEART_RATE_MAX = 175
local HEART_RATE_CRITICAL_THRESHOLD = 150
local HEART_RATE_STRESS_FACTOR = 2.8
local HEART_RATE_TEMP_STRESS_CURVE = 0.6
local HEART_RATE_NOISE_FACTOR = 1.5
local HEART_RATE_WALKING_FACTOR = 12.0
local HEART_MONITOR_VOLUME = 0.0175
local HEARTBEAT_B_THRESHOLD = 60
local HEARTBEAT_C_THRESHOLD = 110
local HEARTBEAT_WIDTH = 0.03

local BREATH_VOLUME_MUL = 0.65
local BREATH_B_STRESS_THRESHOLD = 6.5
local BREATH_A_INTERVAL_MIN = 0.25
local BREATH_A_INTERVAL_MAX = 0.6
local BREATH_B_INTERVAL_MIN = 0.1
local BREATH_B_INTERVAL_MAX = 0.3
local BREATH_C_INTERVAL_MIN = 0.0
local BREATH_C_INTERVAL_MAX = 0.05
local GASP_DELAY_MIN = 0.25
local GASP_DELAY_MAX = 0.35
local GASP_VOLUME_MIN = 1.75
local GASP_VOLUME_MAX = 2.5

local VICTIM_ESCAPE_DISTANCE = 130
local VICTIM_SPEED = 1.0
local VICTIM_FOOTSTEP_VOLUME = 0.5
local VICTIM_STATIONARY_STRESS_RATE_A = 0.02
local VICTIM_STATIONARY_STRESS_RATE_B = 0.07
local VICTIM_STOP_TEMP_STRESS_MIN = 1.75
local VICTIM_STOP_TEMP_STRESS_MAX = 3.5
local VICTIM_STOP_STRESS_MIN = 1.0
local VICTIM_STOP_STRESS_MAX = 2.5
local VICTIM_TEMP_STRESS_MAX = 20.0
local VICTIM_TEMP_STRESS_DECAY_RATE = 0.75
local VICTIM_SHOCKING_STRESS_MIN = 4.0
local VICTIM_TEMP_STRESS_SPILLOVER_RATE = 0.0125
local VICTIM_TEMP_STRESS_DECAY_ATTEN = 0.75
local VICTIM_TEMP_STRESS_DECAY_STRESS_POWER_UNIT = 5.0
local VICTIM_REMARK_ALMOST_THERE_THRESHOLD = 20
local VICTIM_BRAVERY_INITIAL = 10.0
local VICTIM_BRAVERY_USAGE_RATE = 1.0
local VICTIM_BRAVERY_EFFICIENCY_A = 1.0
local VICTIM_BRAVERY_EFFICIENCY_B = 0.75
local VICTIM_BRAVERY_INEFFICIENT_STRESS = 15.0

local VO_DONOVET_VOLUME = 0.125
local VO_VICTIM_VOLUME = 0.2
local VO_COMPUTER_VOLUME = 0.075
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
    [130] = "$redgreen/vo/computer_130",
    [140] = "$redgreen/vo/computer_140",
    [150] = "$redgreen/vo/computer_150",
    [160] = "$redgreen/vo/computer_160",
    [170] = "$redgreen/vo/computer_170",
    [180] = "$redgreen/vo/computer_180",
    [190] = "$redgreen/vo/computer_190",
    [200] = "$redgreen/vo/computer_200"
}

local MONSTER_STATE_IDLE = 'idle'
local MONSTER_STATE_MENACE = 'menace'
local MONSTER_STATE_ATTACK = 'attack'

local MONSTER_IDLE_MIN_TIME_A = 20.0
local MONSTER_IDLE_MIN_TIME_B = 3.2
local MONSTER_IDLE_MAX_TIME = 30.0
local MONSTER_IDLE_INTERVAL = 3.23
local MONSTER_IDLE_INTERVAL_P = 0.19

local MONSTER_MENACE_DELAY_A = 4.0
local MONSTER_MENACE_DELAY_B = 3.0
local MONSTER_MENACE_DELAY_CURVE = 2.0
local MONSTER_MENACE_MIN_TIME = 5.6
local MONSTER_MENACE_MAX_TIME = 16.5
local MONSTER_MENACE_CRITICAL_DETECT_INTERVAL = 5.2
local MONSTER_MENACE_CRITICAL_DETECT_INTERVAL_P = 0.2125
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
    --- Is the soundscape currently supposed to be active?
    soundscape_active = true,
    --- @class RedGreenVictim
    victim = {
        --- Victim's current heart rate
        heart_rate = HEART_RATE_BASE,
        --- Victim's current persistent stress (Ps) level
        stress = 0.0,
        --- Victim's bravery (Br) level. Bravery removes Ts over time and does not replenish.
        bravery = VICTIM_BRAVERY_INITIAL,
        --- Victim's temporary stress (Ts) level
        temp_stress = 0.0,
        --- Victim's distance from the exit
        goal_distance = VICTIM_ESCAPE_DISTANCE,
        --- Last distance that the computer VO reported
        last_reported_distance = VICTIM_ESCAPE_DISTANCE,
        --- Is heart monitor running?
        ekg_enabled = false,
        --- Is the heart monitor FREAKING OUT?
        ekg_panic_mode = false,
        --- Is the victim breathing audible?
        breath_enabled = false,
        --- Is the victim shocked at something? (drives gasp sfx)
        shocked = false,
        --- Is victim currently walking?
        walking = false,
        --- Has the victim moved at all since the game started?
        has_moved = false,
        --- Has the victim stopped at all since the game started?
        has_stopped = false
    },
    --- @class RedGreenMonster
    monster = {
        --- Is the monster AI active?
        active = false,
        --- @type Fsm
        ai = nil,
        --- Is the monster vocalizing?
        vocals_enabled = false,
    }
}

function game.victim:decay_temp_stress(delta_time, modifier)
    self.temp_stress = math.clamp(self.temp_stress - VICTIM_TEMP_STRESS_DECAY_RATE * modifier * delta_time, 0.0, VICTIM_TEMP_STRESS_MAX)
end

function game.victim:update_bravery(delta_time)
    -- Bravery decay
    if self.temp_stress > 0 then
        local efficiency = math.remap(self.stress, 0.0, VICTIM_BRAVERY_INEFFICIENT_STRESS, VICTIM_BRAVERY_EFFICIENCY_A, VICTIM_BRAVERY_EFFICIENCY_B, true)
        local bravery_usage_max = math.min(self.bravery, delta_time * VICTIM_BRAVERY_USAGE_RATE)
        local temp_stress_drain_max = math.min(self.temp_stress, bravery_usage_max * efficiency)
        local bravery_usage = temp_stress_drain_max / efficiency
        self.temp_stress = math.max(0.0, self.temp_stress - temp_stress_drain_max)
        self.bravery = math.max(0.0, self.bravery - bravery_usage)
    end
end

--- @param amount number
--- @param allow_shock boolean?
function game.victim:add_temp_stress(amount, allow_shock)
    if allow_shock == nil then
        allow_shock = true
    end

    if allow_shock and amount >= VICTIM_SHOCKING_STRESS_MIN then
        self.shocked = true
    end

    module:log("Victim: " .. string.format("%+.1f", amount) .. " Ts.")

    self.temp_stress = math.clamp(self.temp_stress + amount, 0.0, VICTIM_TEMP_STRESS_MAX)
end

function game.victim:add_stress(amount, is_accum)
    if amount >= VICTIM_SHOCKING_STRESS_MIN then
        self.shocked = true
    end

    if not is_accum then
        module:log("Victim: " .. string.format("%+.1f", amount) .. " Ps.")
    end

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
    return game.victim.walking
end

--- @type FsmStateTable
local MONSTER_STATES = {
    [MONSTER_STATE_IDLE] = function(self, from_state)
        game.monster.vocals_enabled = false
        local victim_distance_percent = math.invlerp(game.victim.goal_distance, 0.0, VICTIM_ESCAPE_DISTANCE)
        local wait_time_min = math.lerp(MONSTER_IDLE_MIN_TIME_A, MONSTER_IDLE_MIN_TIME_B, 1.0 - (victim_distance_percent ^ 2))
        agent.wait(wait_time_min)
        local timeout = MONSTER_IDLE_MAX_TIME - wait_time_min
        agent.chance_interval(MONSTER_IDLE_INTERVAL, MONSTER_IDLE_INTERVAL_P, timeout)
        self:transition(MONSTER_STATE_MENACE)
    end,
    [MONSTER_STATE_MENACE] = function(self)
        game.monster.vocals_enabled = true
        local victim_distance_percent = math.invlerp(game.victim.goal_distance, 0.0, VICTIM_ESCAPE_DISTANCE)
        local menace_delay = math.lerp(MONSTER_MENACE_DELAY_A, MONSTER_MENACE_DELAY_B, math.pow(victim_distance_percent, MONSTER_MENACE_DELAY_CURVE))
        agent.wait(menace_delay)
        local menace_time = rand_float(MONSTER_MENACE_MIN_TIME, MONSTER_MENACE_MAX_TIME)
        
        agent.multi_task(
            -- Check if the victim is moving; if so, attack
            function()
                if agent.wait_cancel(menace_time, check_victim_detectable) then
                    self:transition(MONSTER_STATE_ATTACK)
                else
                    self:transition(MONSTER_STATE_IDLE)
                end
            end,
            -- Periodically check (with p probability) if the victim's bpm is critical; if so, attack
            function()
                while true do
                    agent.chance_interval(MONSTER_MENACE_CRITICAL_DETECT_INTERVAL, MONSTER_MENACE_CRITICAL_DETECT_INTERVAL_P)
                    if game.victim.heart_rate >= HEART_RATE_CRITICAL_THRESHOLD then
                        self:transition(MONSTER_STATE_ATTACK)
                        break
                    end
                end
            end
        )
    end,
    [MONSTER_STATE_ATTACK] = function(self)
        game.controls_locked = true
        game.victim.walking = false
        game.victim:add_stress(5)
        game.monster.vocals_enabled = false
        sound.wait(Channel.PHONE05)
        agent.wait(rand_float(0.35, 0.5))
        module:log("Monster: \"bonjour\"")
        sound.play("$redgreen/monster/scream", Channel.PHONE10, { volume = 0.4 })
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
    self.soundscape_active = true
    table.clear(self.stop_digits_used)
    -- reset victim
    self.victim.heart_rate = HEART_RATE_BASE
    self.victim.stress = 0.0
    self.victim.temp_stress = 0.0
    self.victim.bravery = VICTIM_BRAVERY_INITIAL
    self.victim.last_reported_distance = VICTIM_ESCAPE_DISTANCE
    self.victim.goal_distance = VICTIM_ESCAPE_DISTANCE
    self.victim.ekg_enabled = false
    self.victim.ekg_panic_mode = false
    self.victim.breath_enabled = false
    self.victim.shocked = false
    self.victim.walking = false
    self.victim.has_moved = false
    self.victim.has_stopped = false
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

    local victim = game.victim

    sound.play("$redgreen/ambient/amb_dungeon", Channel.BG01, { looping = true, skip = rand_float(0, 30), volume = 0.15 })

    agent.do_task_while(
        function()
        agent.multi_task(
                -- Water drops
                function()
                    local first_moment = true

                    while true do
                        agent.wait(first_moment and rand_float(0, 25) or rand_float(5, 30))
                        sound.play("$redgreen/ambient/moment_drip_*", Channel.BG02, { volume = rand_float(0.01, 0.1), speed = rand_float(0.8, 1.15), interrupt = false })
                        first_moment = false
                    end
                end,
                -- Rare moments
                function()
                    while true do
                        agent.wait(rand_float(5, 18))
                        if not sound.is_busy(Channel.BG03) and chance(0.375) then
                            sound.play_wait("$redgreen/ambient/moment_rare_*", Channel.BG03, { volume = rand_float(0.005, 0.125), speed = rand_float(0.9, 1.15) })
                        end
                    end
                end,
                -- Screams
                function()
                    local monster = game.monster
                    while true do
                        agent.wait(rand_float(12, 45))
                        if not game.controls_locked and not sound.is_busy(Channel.BG03) and chance(0.055) then
                            sound.play("$redgreen/ambient/moment_scream_*", Channel.BG03, { volume = rand_float(0.1, 0.2), speed = rand_float(0.9, 1.1) })
                            victim.shocked = true
                            module:log("[Distant scream]")
                            victim:add_temp_stress(3.0)
                            local monster_state = monster.ai:state()
                            if monster_state == MONSTER_STATE_MENACE or monster_state == MONSTER_STATE_IDLE then
                                monster.ai:transition(MONSTER_STATE_IDLE)
                            end
                        end
                    end
                end,
                -- Phantom footsteps
                function()
                    while true do
                        agent.wait(rand_float(8, 20))
                        if chance(0.05 + 0.025 * math.clamp(victim.stress + victim.temp_stress, 0, 5)) and not game.controls_locked and victim.walking then
                            local interval_modifier = rand_float(0.75, 1.2)
                            local pitch_modifier = rand_float(0.8, 1.1)
                            local volume_atten_accum = rand_float(0.5, 1.0)
                            local volume_atten_delta = rand_float(-0.15, -0.05)
                            local volume_buildup_accum = rand_float(0.05, 0.125)
                            local volume_buildup_delta = rand_float(0.125, 0.35)
                            local victim_heard = false
                            module:log("[Phantom footsteps approach!]")
                            for i = 1, rand_int_bias_high(6, 16) do
                                if not victim.walking then
                                    module:log("[Phantom footsteps interrupted.]")
                                    break 
                                end
                                local volume_modifier = (volume_buildup_accum ^ 2) * (volume_atten_accum ^ 0.5) * 2
                                sound.play("$redgreen/footstep_*", Channel.BG04, {
                                    volume = rand_float(0.4, 0.6) * PHANTOM_FOOTSTEP_VOLUME * volume_modifier,
                                    speed = pitch_modifier * PHANTOM_FOOTSTEP_PLAYBACK_SPEED,
                                    interrupt = true
                                })
                                agent.wait(rand_float(0.19, 0.23) * interval_modifier)
                                volume_buildup_accum = math.clamp(volume_buildup_accum + volume_buildup_delta, 0, 1)
                                volume_atten_accum = math.clamp(volume_atten_accum + volume_atten_delta, 0, 1)

                                if not victim_heard and volume_modifier > PHANTOM_FOOTSTEP_SENSITIVITY then
                                    victim_heard = true
                                    local stress_penalty = rand_float(PHANTOM_FOOTSTEP_TEMP_STRESS_MIN, PHANTOM_FOOTSTEP_TEMP_STRESS_MAX)
                                    victim:add_temp_stress(stress_penalty)
                                    module:log("Victim: Footsteps heard. Stress increasing.")
                                end
                            end
                        end
                    end
                end
            )
        end,
        function() return game.soundscape_active end
    )
    sound.fade_out_multi(ALL_BG_CHANNELS, 0.85)
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
                local static_volume_noise_scale = 1.0 - static_volume_noise * MONSTER_MENACE_STATIC_VOLUME_NOISE_SCALE

                if game.monster.vocals_enabled then
                    static_volume = math.step_to(static_volume, 1.0, dt * MONSTER_MENACE_STATIC_FADEIN_RATE)
                    vocal_volume = math.step_to(vocal_volume, 1.0, dt * MONSTER_MENACE_VOCAL_FADEIN_RATE)
                else
                    static_volume = math.step_to(static_volume, 0.0, dt * MONSTER_MENACE_STATIC_FADEOUT_RATE)
                    vocal_volume = math.step_to(vocal_volume, 0.0, dt * MONSTER_MENACE_VOCAL_FADEOUT_RATE)
                end

                sound.set_channel_volume(Channel.PHONE04, math.clamp(static_volume * static_volume_noise_scale, 0, 1))
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
        while victim.ekg_enabled do
            if victim.ekg_panic_mode then 
                gpio.write_pin(OUT_VIBRATE, GPIO_HIGH)
                agent.yield()
            else 
                gpio.write_pin(OUT_VIBRATE, GPIO_HIGH)
                sound.play(select_heartbeat_bank(victim.heart_rate), Channel.PHONE07, { speed = rand_float(0.9, 1.1), interrupt = true })
                local beep_speed = (victim.heart_rate > HEART_RATE_CRITICAL_THRESHOLD and 1.1 or 1.0) * rand_float(0.9985, 1.0015)
                sound.play("$redgreen/heart_monitor_beep", Channel.PHONE08, { volume = HEART_MONITOR_VOLUME, speed = beep_speed, interrupt = true })
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

    while true do
        local time = engine_time()
        local noise = perlin_sample(time, 0, 3, 1, 0.5, 2.0, noise_seed)

        -- This basically means that more temp stress => more influence on BPM
        local temp_stress_curved = math.pow(victim.temp_stress / VICTIM_TEMP_STRESS_MAX, HEART_RATE_TEMP_STRESS_CURVE) * VICTIM_TEMP_STRESS_MAX

        -- BPM offset from stress gauges
        local bpm_offset_stress = (victim.stress + temp_stress_curved) * HEART_RATE_STRESS_FACTOR
        -- BPM offset from BPM noise
        local bpm_offset_noise = noise * HEART_RATE_NOISE_FACTOR
        -- BPM offset from victim walking state
        local bpm_offset_walking = victim.walking and HEART_RATE_WALKING_FACTOR or 0.0

        -- Offsets summed up to produce final BPM
        victim.heart_rate = math.clamp(HEART_RATE_BASE + bpm_offset_stress + bpm_offset_noise + bpm_offset_walking, HEART_RATE_MIN, HEART_RATE_MAX)
        
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
                    sound.play("$redgreen/vo/computer_go", Channel.PHONE06, { volume = VO_COMPUTER_VOLUME, interrupt = true })

                    -- victim reaction
                    if not victim.has_moved then
                        agent.wait(0.35)
                        local prev_breath_state = victim.breath_enabled
                        victim.breath_enabled = false
                        sound.fade_out(Channel.PHONE02, 0.25)
                        sound.play("$redgreen/vo/victim_okay", Channel.PHONE02, { volume = VO_VICTIM_VOLUME, interrupt = false })
                        agent.wait(0.5)
                        victim.breath_enabled = prev_breath_state
                    end

                    victim.walking = true
                    victim.has_moved = true
                    module:log("Victim: Moving.")
                elseif victim.walking then
                    -- stop
                    if not game.stop_digits_used[digit] then
                        -- allow stop
                        game.stop_digits_used[digit] = true
                        sound.play("$redgreen/vo/computer_stop", Channel.PHONE06, { volume = VO_COMPUTER_VOLUME, interrupt = true })

                        -- victim reaction
                        if not victim.has_stopped then
                            local prev_breath_state = victim.breath_enabled
                            victim.breath_enabled = false
                            sound.fade_out(Channel.PHONE02, 0.25)
                            sound.play("$redgreen/vo/victim_reaction_stopped_*", Channel.PHONE02, { volume = VO_VICTIM_VOLUME, delay = rand_float(0.1, 0.2), interrupt = false })
                            victim.breath_enabled = prev_breath_state
                        end

                        victim.walking = false
                        victim.has_stopped = true
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

--- @async
local function task_scenario_win()
    module:log("Victim escaped!")
    local victim = game.victim
    game.monster.active = false
    game.monster.vocals_enabled = false
    game.controls_locked = true
    victim.walking = false
    victim:add_temp_stress(15.0)
    sound.play_wait("$redgreen/escape_door", Channel.PHONE09, { volume = 0.3 }, -2.0)
    game.soundscape_active = false
    agent.wait(2.0)
    victim.ekg_enabled = false
    agent.wait(1.25)
    sound.play_wait("$redgreen/vo/computer_congrats", Channel.PHONE06, { volume = VO_COMPUTER_VOLUME })
    victim.breath_enabled = false
    agent.wait(2.0)
    agent.end_call()
end

--- @param wait boolean?
--- @param wait_time_offset number?
local function report_victim_distance(wait, wait_time_offset)
    local report_sound_path = VO_COMPUTER_DISTANCE_LINES[game.victim.last_reported_distance]
    if report_sound_path then
        local success, length = sound.play(report_sound_path, Channel.PHONE06, { volume = VO_COMPUTER_VOLUME, interrupt = false })
        if wait then
            if length and wait_time_offset then
                sound.wait(length + wait_time_offset)
            else
                sound.wait(Channel.PHONE06)
            end
        end
    end
end

--- @return string @ The selected sound bank name
--- @return number @ The interval time in seconds to wait before playing the breath
--- @return number @ The volume multiplier for the breath
--- @return number @ The speed multiplier for the breath
local function select_victim_breath_params(stress, temp_stress, heart_bpm)
    local combined_stress = stress + temp_stress
    local volume_influence = math.invlerp(temp_stress, 0, VICTIM_TEMP_STRESS_MAX) ^ 2
    if heart_bpm >= HEART_RATE_CRITICAL_THRESHOLD then
        return "$redgreen/vo/victim_breath_c_*", 
        rand_float(BREATH_C_INTERVAL_MIN, BREATH_C_INTERVAL_MAX), 
        math.lerp(1.0, rand_float(1.35, 2.25), volume_influence),
        rand_float(0.98, 1.05)
    elseif combined_stress > BREATH_B_STRESS_THRESHOLD then
        return "$redgreen/vo/victim_breath_b_*", 
        rand_float(BREATH_B_INTERVAL_MIN, BREATH_B_INTERVAL_MAX), 
        math.lerp(1.0, rand_float(1.35, 2.25), volume_influence),
        rand_float(0.9, 1.1)
    else
        return "$redgreen/vo/victim_breath_a_*", 
        rand_float(BREATH_A_INTERVAL_MIN, BREATH_A_INTERVAL_MAX), 
        rand_float(0.8, 1.3),
        rand_float(0.9, 1.1)
    end
end

--- @async
local function task_victim_breathing_speech()
    local victim = game.victim
    local function check_breath_interruptable() return victim.shocked end

    while true do
        if victim.breath_enabled then
            if victim.shocked then
                victim.shocked = false
                local gasp_volume = rand_float(GASP_VOLUME_MIN, GASP_VOLUME_MAX)
                --agent.wait(rand_float(GASP_DELAY_MIN, GASP_DELAY_MAX))
                sound.fade_out(Channel.PHONE02, rand_float(GASP_DELAY_MIN, GASP_DELAY_MAX))
                sound.play_wait("$redgreen/vo/victim_gasp_*", Channel.PHONE02, { volume = VO_VICTIM_VOLUME * gasp_volume * BREATH_VOLUME_MUL, speed = rand_float(0.9, 1.1), interrupt = true })
            else
                local breath_bank, breath_interval, breath_volume, breath_speed = select_victim_breath_params(victim.stress, victim.temp_stress, victim.heart_rate)
        
                if not agent.wait_cancel(breath_interval, check_breath_interruptable) then
                    sound.play(breath_bank, Channel.PHONE02, { volume = VO_VICTIM_VOLUME * breath_volume * BREATH_VOLUME_MUL, speed = breath_speed })
                    while sound.is_busy(Channel.PHONE02) and not check_breath_interruptable() do
                        agent.yield()
                    end
                end
            end
        else
            agent.yield()
        end
    end
end

--- @async
local function task_update_victim()
    local victim = game.victim
    local last_tick_time = engine_time()
    local last_report_time = engine_time()
    while true do
        if victim.walking and victim.goal_distance <= 0.0 and game.monster.ai:state() == MONSTER_STATE_IDLE then
            task_scenario_win()
        end

        local time = engine_time()
        local dt = time - last_tick_time
        last_tick_time = time

        if not game.controls_locked then
            if victim.walking then
                local distance_delta = VICTIM_SPEED * dt
                local distance_prev = victim.goal_distance
                local distance_updated = math.max(0, distance_prev - distance_delta)
                victim.goal_distance = distance_updated
    
                -- Quick and dirty log of victim distance
                if math.abs(math.ceil(distance_prev) - math.ceil(distance_updated)) >= 1 then
                    if victim:update_distance_report() then
                        report_victim_distance()
                    end
                    module:log("" .. math.ceil(distance_updated) .. "m from exit.")
                end
            else
                local stationary_stress_rate = math.remap(victim.goal_distance, VICTIM_ESCAPE_DISTANCE, 0.0, VICTIM_STATIONARY_STRESS_RATE_A, VICTIM_STATIONARY_STRESS_RATE_B, true)
                victim:add_stress(dt * stationary_stress_rate, true)
            end
            
            victim:add_stress(victim.temp_stress * VICTIM_TEMP_STRESS_SPILLOVER_RATE * dt, true)
        end
        
        if not victim.walking then
            local temp_stress_decay_modifier = math.pow(VICTIM_TEMP_STRESS_DECAY_ATTEN, victim.stress / VICTIM_TEMP_STRESS_DECAY_STRESS_POWER_UNIT)
            victim:decay_temp_stress(dt, temp_stress_decay_modifier)
        end

        victim:update_bravery(dt)

        if victim.ekg_enabled and time - last_report_time > 2.5 then
            last_report_time = time
            local report_bpm = math.floor(victim.heart_rate)
            module:log(string.format("Ps = %.1f | Ts = %.1f | Br = %.1f | ♥ = %d%s", victim.stress, victim.temp_stress, victim.bravery, victim.heart_rate, (victim.heart_rate >= HEART_RATE_CRITICAL_THRESHOLD and " (!)" or "")))
        end
        
        agent.yield()
    end
end

--- @async
local function task_victim_remarks()
    local victim = game.victim
    local remarked_almost_there = false
    while true do
        if not game.controls_locked then
            if not remarked_almost_there and victim.last_reported_distance <= VICTIM_REMARK_ALMOST_THERE_THRESHOLD and victim.walking then
                remarked_almost_there = true
                victim.breath_enabled = false
                sound.fade_out(Channel.PHONE02, 0.25)
                sound.play("$redgreen/vo/victim_reaction_almost_there", Channel.PHONE02, { volume = VO_VICTIM_VOLUME })
                victim.breath_enabled = true
            end
        end
        agent.yield()
    end
end

--- @async
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

--- @async
local function task_intro()
    local victim = game.victim
    agent.wait(3)

    -- Intro dialogue
    sound.play_wait("$redgreen/vo/intro/01_victim", Channel.PHONE02, { volume = VO_VICTIM_VOLUME }, -1.75)
    sound.play_wait("$redgreen/vo/intro/02_donovet", Channel.PHONE01, { volume = VO_DONOVET_VOLUME }, -2.1)
    sound.play_wait("$redgreen/vo/intro/03_victim", Channel.PHONE02, { volume = VO_VICTIM_VOLUME }, -1.8)
    game.victim:add_temp_stress(20.0, false)
    game.victim.breath_enabled = true
    sound.play_wait("$redgreen/vo/intro/04_donovet_01", Channel.PHONE01, { volume = VO_DONOVET_VOLUME }, -4.5)
    victim.breath_enabled = false
    sound.fade_out(Channel.PHONE02, 0.25)
    sound.play("$redgreen/vo/intro/04_victim_reaction", Channel.PHONE02, { volume = VO_VICTIM_VOLUME })
    victim.breath_enabled = true
    sound.play_wait("$redgreen/vo/intro/04_donovet_02", Channel.PHONE01, { volume = VO_DONOVET_VOLUME })
    sound.play_wait("$redgreen/vo/computer_monitor_ready", Channel.PHONE06, { volume = VO_COMPUTER_VOLUME })
    game.victim.ekg_enabled = true
    agent.wait(0.25)
    game.victim:add_temp_stress(10.0, false)
    sound.play_wait("$redgreen/vo/intro/05_donovet", Channel.PHONE01, { volume = VO_DONOVET_VOLUME }, -2.0)
    game.victim:add_temp_stress(15.0, false)
    game.victim.breath_enabled = false
    agent.wait(1.0)
    sound.fade_out(Channel.PHONE02, 0.2)
    sound.play_wait("$redgreen/vo/intro/06_victim", Channel.PHONE02, { volume = VO_VICTIM_VOLUME }, -1.25)
    sound.play_wait("$redgreen/vo/intro/07_donovet", Channel.PHONE01, { volume = VO_DONOVET_VOLUME }, 0.0)
    sound.play_wait("$redgreen/vo/computer_measuring_distance", Channel.PHONE06, { volume = VO_COMPUTER_VOLUME }, 0)
    report_victim_distance(true, -1.0)
    sound.play_wait("$redgreen/vo/intro/08_victim", Channel.PHONE02, { volume = VO_VICTIM_VOLUME })

    -- Start the game
    game.controls_locked = false
    game.monster.active = true
    game.victim.ekg_enabled = true
    game.victim.breath_enabled = true
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
            task_victim_breathing_speech,
            task_victim_remarks,
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