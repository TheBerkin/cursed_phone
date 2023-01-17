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

--- @alias GpioPull string

--- Indicates no pull resistor to be activated on an input.
--- @type GpioPull
GPIO_PULL_NONE = "none"

--- Indicates to activate the built-in pull-up resistor on an input.
--- @type GpioPull
GPIO_PULL_UP = "up"

--- Indicates to activate the built-in pull-down resistor on an input.
--- @type GpioPull
GPIO_PULL_DOWN = "down"


--- @alias GpioLogicLevel boolean

--- HIGH logic level.
--- @type GpioLogicLevel
GPIO_HIGH = true

--- LOW logic level.
--- @type GpioLogicLevel
GPIO_LOW = false

NATIVE_API(function()
    gpio = {}

    --- Registers a pin as an input pin.
    --- @param pin integer @ The pin to register
    --- @param pull GpioPull? @ The pull resistor to activate (`GPIO_PULL_*`), defaults to `GPIO_PULL_NONE`
    --- @param debounce_time number? @ The debounce time in seconds
    function gpio.register_input(pin, pull, debounce_time) end

    --- Registers a pin as an output pin.
    --- @param pin integer
    function gpio.register_output(pin) end

    --- Reads the logic level from an input pin.
    --- @param pin integer
    --- @return GpioLogicLevel
    function gpio.read_pin(pin) return GPIO_LOW end

    --- Sets the logic level on an output pin.
    --- @param pin integer
    --- @param value GpioLogicLevel
    function gpio.write_pin(pin, value) end

    --- Unregisters a previously registered pin.
    --- @param pin integer
    function gpio.unregister(pin) end

    --- Unregisters all registered pins.
    function gpio.unregister_all() end
end)