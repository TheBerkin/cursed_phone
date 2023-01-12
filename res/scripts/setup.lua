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

--- Stub function to represent a table of native functions without initializing it.
--- @param api_defs function
function NATIVE_API(api_defs) 
    if __STUB == true then
        api_defs()
    end
end

function empty_func() end

print_info()