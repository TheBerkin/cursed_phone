--- Special state key used to mark the exit state of a state machine.
FSM_EXIT = {}

local ACTIVE_FSM_COROUTINES = {}
setmetatable(ACTIVE_FSM_COROUTINES, { __mode = 'k' })

local function is_fsm_context()
    local context, is_main = coroutine.running()
    return ACTIVE_FSM_COROUTINES[context] ~= nil and not is_main
end

--- @alias FsmStateTable table<any, FsmState|async fun(self: Fsm, from_state_key: any, ...)>

--- A function table for a specific state in a finite state machine.
--- @class FsmState
--- @field enter async fun(self: Fsm, from_state_key: any?, ...)?
--- @field exit async fun(self: Fsm)?

--- A Finite State Machine (FSM) runnable by agent tasks.
--- @class Fsm
--- @field package _last_response_code IntentResponseCode
--- @field package _last_response_data any?
--- @field package _coroutine thread
--- @field package _context any
--- @field package _state_key any
--- @field package _state_table FsmStateTable
--- @field package _on_transition fun(fsm: Fsm, from: any?, to: any?, ...)?
local C_Fsm = {}

local M_Fsm = {
    __index = C_Fsm
}

--- Generates the coroutines used by the FSM executor.
--- @return thread
local function gen_transition_coroutine(fsm, prev_state_key, next_state_key, ...)
    local params = table.pack(...)
    local co = coroutine.create(function()
        local prev_state = fsm._state_table[prev_state_key]
        local next_state = fsm._state_table[next_state_key]

        local next_state_type = type(next_state)

        local fn_prev_exit = type(prev_state) == 'table' and prev_state.exit or nil
        local fn_next_enter = nil

        if next_state_type == 'table' then
            fn_next_enter = next_state and next_state.enter or nil
        elseif next_state_type == 'function' then
            fn_next_enter = next_state
        end

        if fn_prev_exit then
            fn_prev_exit(fsm)
        end

        if fn_next_enter then
            fn_next_enter(fsm, prev_state_key, table.unpack(params))
        end

        -- Don't let the coroutine die until a transition happens
        if next_state_key ~= FSM_EXIT then
            while true do
                task.yield()
            end
        end
    end)
    ACTIVE_FSM_COROUTINES[co] = true
    return co
end

--- Creates a new finite state machine.
--- @param state_table FsmStateTable @ The table of states to use.
--- @param init_state any? @ The initial state to set.
--- @param context any? @ A context object to associate with the FSM.
--- @return Fsm
function Fsm(state_table, init_state, context)
    local fsm = {
        _state_table = state_table,
        _state_key = init_state
    }
    --- @cast fsm Fsm
    setmetatable(fsm, M_Fsm)
    fsm._state_key = init_state
    fsm._coroutine = gen_transition_coroutine(fsm, nil, init_state)
    fsm._context = context
    return fsm
end

--- Sets the handler function that fires when a state transition occurs.
--- @param handler fun(fsm: Fsm, from: any, to: any)
function C_Fsm:on_transition(handler)
    assert(type(handler) == 'function', 'handler must be a function')
    self._on_transition = handler
end

--- Gets the context object associated with the FSM.
--- @return any
function C_Fsm:context()
    return self._context
end

--- Sets the context object associated with the FSM.
function C_Fsm:set_context(context)
    self._context = context
end

--- @async
--- Asynchronously runs the state machine. It is not guaranteed to exit.
function C_Fsm:run()
    while self:is_active() do
        self:tick()
    end
end

--- @async
--- Asynchronously advances the state machine by one tick.
function C_Fsm:tick()
    local success, intent_code, intent_data, continuation = coroutine.resume(self._coroutine, self._last_response_code, self._last_response_data)
    if not success then
        task.yield()
        return false
    end
    self._last_response_code, self._last_response_data = task.intent(intent_code, intent_data, continuation)
    return true
end

--- Returns a function that calls the `run` method for this FSM.
--- @return async fun()
function C_Fsm:wrap()
    return function() self:run() end
end

--- @async
--- Transitions the FSM to the specified state key. Yields if called inside a state.
--- @param to_state_key any @ The key of the state to transition to.
--- @param ... any @ Optional additional arguments to pass to the state.
--- @return boolean
function C_Fsm:transition(to_state_key, ...)
    local called_by_fsm = is_fsm_context()

    if to_state_key == nil then return false end
    local from_state_key = self._state_key
    self._state_key = to_state_key
    self._coroutine = gen_transition_coroutine(self, from_state_key, to_state_key, ...)

    if self._on_transition then
        self._on_transition(self, from_state_key, to_state_key, ...)
    end

    if called_by_fsm then task.yield() end
    return true
end

--- Returns the current state's key.
function C_Fsm:state()
    return self._state_key
end

--- Transitions the FSM to the exit state next time it's ticked.
function C_Fsm:exit_next()
    self:transition(FSM_EXIT)
end

--- Returns a boolean indicating whether the state machine is in-progress.
--- @return boolean
function C_Fsm:is_active()
    return self._coroutine and coroutine.status(self._coroutine) ~= 'dead'
end