local module = create_agent("intercept", nil, AGENT_ROLE_INTERCEPT)

local MAX_MESSAGE_TIME = 30

module:set_ringback_enabled(false)

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

local vsc_handlers = {
    -- Last-Call Return
    ["69"] = function(self) 
        local last_call_return_id = phone.last_caller_id()
        if last_call_return_id then
            print("Returning last call to Agent ID: " .. last_call_return_id)
        else
            print("No previous caller available for callback")
        end
        --- @cast last_call_return_id integer
        agent.forward_call_id(last_call_return_id)
    end,
    -- Adjust Volume
    ["11"] = function(self, phone_number)
        if #phone_number == 0 then return end
        local volume_raw = tonumber(string.sub(phone_number, 1, 1))
        if not volume_raw then
            return 
        end
        local volume = volume_raw / 9.0
        print("Adjusting volume to " .. (volume * 100) .. "%")
        sound.set_master_volume(volume)
        sound.play("music/holding02", Channel.PHONE01, { looping = true })
        agent.wait(20)
        sound.fade_out(Channel.PHONE01, 10)
        agent.end_call()
    end
}

local reason_handlers = {
    -- Number is invalid or a vertical service code
    [CALL_REASON_NUMBER_REDIRECTED] = function(self)
        local vsc
        local phone_number = agent.caller_dialed_number()
        local vsc_handled = false
        repeat
            vsc, phone_number = split_vsc(phone_number)
            if vsc then
                print("VSC " .. vsc)
                local vsc_handler = vsc_handlers[vsc]
                if vsc_handler then
                    vsc_handler(self, phone_number)
                end
                vsc_handled = true
            end
        until (not vsc)
        
        if vsc_handled and phone_number and #phone_number > 0 then
            agent.forward_call(phone_number)
        end

        sound.play_special_info_tone(SpecialInfoTone.INTERCEPT)
        sound.wait(CHAN_SIGIN)
        agent.wait(0.05)
        sound.play_wait("intercept/intercept_disconnected_*", CHAN_PHONE1)
        sound.play_fast_busy_tone()
        agent.wait()
    end,

    -- Phone was left off the hook
    [CALL_REASON_OFF_HOOK] = function(self)
        local cancel_func = function()
            return call_time() > MAX_MESSAGE_TIME
        end

        while not cancel_func() do
            sound.play_wait_cancel("intercept/intercept_timeout_message_01", CHAN_PHONE1, cancel_func)
            agent.wait_cancel(10, cancel_func)
        end

        sound.play_off_hook_tone()
        agent.wait()
    end
}

-- Immediately answer calls
module:state(AGENT_STATE_CALL_IN, {
    enter = function(self)
        agent.accept_call()
    end
})

-- Call handler for intercept reason
module:state(AGENT_STATE_CALL, {
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

return module