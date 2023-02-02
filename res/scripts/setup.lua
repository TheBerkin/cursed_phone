--[[

    /==========================================================================\
    |========================= CURSED PHONE API FILE ==========================|
    |==========================================================================|
    | This script is required by phone services in order to function properly. |
    | Unless you are making changes to the engine, do not modify this file.    |
    \==========================================================================/

]]

function print_info()
    local is_luajit = type(jit) == 'table'
    if is_luajit then
        print("Running " .. _VERSION .. ", " .. jit.version .. "(" .. jit.arch .. ")")
    else
        print("Running " .. _VERSION .. "")
    end
end

function coerce_boolean(value)
    return not not value
end

function stub() end

print_info()