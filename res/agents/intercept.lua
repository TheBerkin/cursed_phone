local agent = AgentModule("intercept", nil, AgentRole.INTERCEPT)

local MAX_MESSAGE_TIME = 30

agent:set_ringback_enabled(false)

function split_vsc(phone_number)
    if not phone_number then return nil, nil end
    local prefix = phone.is_rotary() and '11' or '%*'
    local vsc_start, vsc_end = string.find(phone_number, '^' .. prefix .. '%d%d')
    if vsc_start and vsc_start > 0 then
        return string.sub(phone_number, vsc_end - 1, vsc_end), string.sub(phone_number, vsc_end + 1, #phone_number)
    else
        return nil, phone_number
    end
end

--- @type table<string, fun(self: AgentModule, phone_number: string?)>
local vsc_handlers = {
    -- Adjust Volume
    ["11"] = function(self, phone_number)
        sound.play("music/holding02", Channel.PHONE01, { looping = true, fadein = 1 })
        while true do 
            local volume_raw = tonumber(task.read_digit())
            if volume_raw then
                local volume = volume_raw / 9.0
                log.info("Adjusting volume to " .. (volume * 100) .. "%")
                sound.set_master_volume(volume)
            end
        end
    end,
    -- Last-Call Return
    ["69"] = function(self) 
        local last_call_return_id = phone.last_caller_id()
        if last_call_return_id then
            log.info("Returning last call to Agent ID: " .. last_call_return_id)
        else
            log.warn("No previous caller available for callback")
        end
        --- @cast last_call_return_id integer
        task.forward_call_id(last_call_return_id)
    end,
}

--- @type table<CallReason, async fun(self: AgentModule)>
local reason_handlers = {
    -- Number is invalid or a vertical service code
    [CallReason.REDIRECTED] = function(self)
        local vsc
        local phone_number = phone.call_dialed_number()
        local vsc_handled = false
        repeat
            vsc, phone_number = split_vsc(phone_number)
            if vsc then
                log.info("VSC " .. vsc)
                local vsc_handler = vsc_handlers[vsc]
                if vsc_handler then
                    vsc_handler(self, phone_number)
                end
                vsc_handled = true
            end
        until (not vsc)
        
        if vsc_handled and phone_number and #phone_number > 0 then
            task.forward_call(phone_number)
        end

        sound.play_special_info_tone(SpecialInfoTone.INTERCEPT)
        sound.wait(Channel.SIG_IN)
        task.wait(0.05)
        sound.play_wait("intercept/intercept_disconnected_*", Channel.PHONE01)
        sound.play_fast_busy_tone()
        task.wait()
    end,

    -- Phone was left off the hook
    [CallReason.OFF_HOOK] = function(self)
        local cancel_func = function()
            return call_time() > MAX_MESSAGE_TIME
        end

        while not cancel_func() do
            sound.play_wait_cancel("intercept/intercept_timeout_message_01", Channel.PHONE01, cancel_func)
            task.wait_cancel(10, cancel_func)
        end

        sound.play_off_hook_tone()
        task.wait()
    end
}

-- Immediately answer calls
agent:state(AgentState.CALL_IN, {
    enter = function(self)
        task.accept_call()
    end
})

-- Call handler for intercept reason
agent:state(AgentState.CALL, {
    enter = function(self)
    end,
    tick = function(self)
        local reason = self:get_call_reason()
        local handler = reason_handlers[reason]
        if handler then
            handler(self)
        end
    end,
    exit = function(self)
    end
})

return agent