--- Provides a basic synchronous pub/sub interface.
--- @class EventLib
event = {}

--- @alias event.handler fun(...): (boolean?)

--- @type table<any, { handlers: PrioritySet, error_handler: fun(err: string) }>
local EVENT_REGISTRY = {}

local function create_error_handler(event_id)
    return function(err)
        log.error(string.format("(error in handler for event '%s':) %s", event_id, err))
    end
end

--- @param event_handler event.handler
local function run_handler(event_handler, error_handler, ...)
    xpcall(event_handler, error_handler, ...)
end

local function validate_event_id(event_id, level)
    if event_id == nil or event_id == '' then 
        error("Event ID cannot be nil or an empty string", (level or 1) + 1)
    end
end

--- @param event_id any
--- @see event.subscribe
function event.publish(event_id, ...)
    validate_event_id(event_id, 2)
    local reg_event = EVENT_REGISTRY[event_id]
    if not reg_event then return end
    reg_event.handlers:foreach(run_handler, reg_event.error_handler, ...)
end

--- @param event_id any
--- @param handler event.handler
--- @param priority? number
--- @return boolean
function event.subscribe(event_id, handler, priority)
    validate_event_id(event_id, 2)
    local reg_event = EVENT_REGISTRY[event_id]
    if not reg_event then
        reg_event = {
            handlers = PrioritySet(),
            error_handler = create_error_handler(event_id)
        }
        EVENT_REGISTRY[event_id] = reg_event
    end
    return reg_event.handlers:add(handler, priority or 0)
end

--- @param event_id any
--- @param handler event.handler
--- @return boolean
function event.unsubscribe(event_id, handler)
    validate_event_id(event_id, 2)
    local reg_event = EVENT_REGISTRY[event_id]
    if not reg_event then return false end
    return (reg_event.handlers:remove(handler))
end

--- @param event_id any
--- @return boolean
function event.unsubscribe_all(event_id) 
    validate_event_id(event_id, 2)
    local reg_event = EVENT_REGISTRY[event_id]
    if not reg_event then return false end
    reg_event.handlers:clear()
    return true
end