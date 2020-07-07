local S = SERVICE_MODULE("intercept", nil, SERVICE_ROLE_INTERCEPT)

local MAX_MESSAGE_TIME = 30

S:set_ringback_enabled(false)

local reason_handlers = {
    -- Number is disconnected
    [CALL_REASON_NUMBER_DISCONNECTED] = function(self)
        sound.play_special_info_tone(SIT_INTERCEPT)
        sound.wait(CHAN_SIGIN)
        service.wait(0.05)
        sound.play_wait("intercept/intercept_disconnected_*", CHAN_PHONE1)
        sound.play_fast_busy_tone()
        while true do
            service.intent(SERVICE_INTENT_WAIT)
        end
    end,

    -- Phone was left off the hook
    [CALL_REASON_OFF_HOOK] = function(self)
        local cancel_func = function()
            return get_call_time() > MAX_MESSAGE_TIME
        end

        while not cancel_func() do
            sound.play_wait_cancel("intercept/intercept_timeout_message_01", CHAN_PHONE1, cancel_func)
            service.wait_cancel(10, cancel_func)
        end

        sound.play_off_hook_tone()
        while true do
            service.intent(SERVICE_INTENT_WAIT)
        end
    end
}

-- Immediately answer calls
S:state(SERVICE_STATE_CALL_IN, {
    enter = function(self)
        service.accept_call()
    end
})

-- Call handler for intercept reason
S:state(SERVICE_STATE_CALL, {
    enter = function(self)
    end,
    tick = function(self)
        local reason = self:get_reason()
        local handler = reason_handlers[reason]
        if handler then
            handler(self)
        end
    end,
    exit = function(self)
    end
})

return S