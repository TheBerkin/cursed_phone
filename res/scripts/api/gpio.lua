--[[

    /==========================================================================\
    |========================= CURSED PHONE API FILE ==========================|
    |==========================================================================|
    | This script is required by phone services in order to function properly. |
    | Unless you are making changes to the engine, do not modify this file.    |
    \==========================================================================/
    
]]

-- ====================================================
-- ==================== GPIO API ======================
-- ====================================================

--- @alias GpioPull integer

--- Indicates no pull resistor to be activated on an input.
--- @type GpioPull
GPIO_PULL_NONE = 0

--- Indicates to activate the built-in pull-up resistor on an input.
--- @type GpioPull
GPIO_PULL_UP = 1

--- Indicates to activate the built-in pull-down resistor on an input.
--- @type GpioPull
GPIO_PULL_DOWN = 2

NATIVE_API(function()
    gpio = {}

    --- Registers a pin as an input pin.
    --- @param pin integer
    --- @param pull GpioPull|nil
    function gpio.register_input(pin, pull) end

    --- Registers a pin as an output pin.
    --- @param pin integer
    function gpio.register_output(pin) end

    --- Reads the state from an input pin.
    --- @param pin integer
    function gpio.get_pin_state(pin) end

    --- Sets the state on an output pin.
    --- @param pin integer
    --- @param state boolean
    function gpio.set_pin_state(pin, state) end

    --- Unregisters a previously registered pin.
    --- @param pin integer
    function gpio.unregister(pin) end
end)