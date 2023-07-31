--[[

    /==========================================================================\
    |========================= CURSED PHONE API FILE ==========================|
    |==========================================================================|
    | This script is required by the engine in order to function properly.     |
    | Unless you are making changes to the engine, do not modify this file.    |
    \==========================================================================/
    
]]

--- @param obj any
--- @param type_expected luatype
--- @param error_message string?
--- @param level integer?
function assert_type(obj, type_expected, error_message, level)
    local type_actual = type(obj)
    if type_actual ~= type_expected then
        local error_message_footer = string.format("expected: %s\nactual: %s", type_expected, type_actual)
        local error_message_final = error_message and string.format("%s\n%s", error_message, error_message_footer) or error_message_footer
        error(error_message_final, (level or 0) + 1)
    end
end