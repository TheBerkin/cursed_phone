--[[

    /==========================================================================\
    |========================= CURSED PHONE API FILE ==========================|
    |==========================================================================|
    | This script is required by the engine in order to function properly.     |
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