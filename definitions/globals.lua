--- @meta

--- Represents a custom ring pattern that can be assigned to an agent.
--- @class RingPattern

--- Indicates whether the engine has developer features enabled.
--- @type boolean
DEVMODE = nil

--- Gets the number of seconds elapsed since the engine was initialized.
--- @return number
function engine_time() end

--- Gets the number of seconds elapsed since the current call started.
--- Returns 0 if no call is active.
--- @return number
function call_time() end

--- Calculates a 2-dimensional Perlin noise sample corresponding to the specified coordinates and noise parameters.
--- @param x number @ The X coordinate of the noise sample.
--- @param y number @ The Y coordinate of the noise sample.
--- @param octaves integer @ The number of octaves (layers) to add to the noise.
--- @param frequency number @ The number of noise cycles per unit length.
--- @param persistence number @ The amplitude multiplier for each successive octave.
--- @param lacunarity number @ The frequency multiplier for each successive octave.
--- @param seed integer @ The seed of the noise generator.
--- @return number
function perlin_sample(x, y, octaves, frequency, persistence, lacunarity, seed) end

--- @param agent_id integer
--- @param loaded boolean
function set_agent_sounds_loaded(agent_id, loaded) end