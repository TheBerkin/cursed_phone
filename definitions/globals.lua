--- @meta

--- @class metatable: table
--- @field __add? fun(a, b): any                    @ Addition operator (`a + b`)
--- @field __call? fun(t: any, ...)                 @ Controls what happens when `t` is called like a function (e.g. `t()`). Arguments are accessible via `...`.
--- @field __concat? fun(a, b): any                 @ Concatenation operator (`a .. b`)
--- @field __div? fun(a, b): any                    @ Division operator (`a / b`)
--- @field __eq? fun(a, b): boolean                 @ Equality operator (`a == b`)
--- @field __index? table | fun(t: any, k): any     @ Controls the value returned by `t.k`. Use `rawget(t, k)` to skip this metamethod.
--- @field __le? fun(a, b): boolean                 @ Less-than-or-equal operator (`a <= b`)
--- @field __len? fun(t: any): any                  @ Length operator (`#t`)
--- @field __lt? fun(a, b): boolean                 @ Less-than operator (`a < b`)
--- @field __metatable? any                         @ If this has a value then it will be returned when `getmetatable(t)` is called on attached tables.
--- @field __mod? fun(a, b): any                    @ Modulo operator (`a % b`)
--- @field __mode? 'k' | 'v' | 'kv'                 @ Controls whether keys and/or values of attached tables are weak references.
--- @field __mul? fun(a, b): any                    @ Multiplication operator (`a * b`)
--- @field __newindex? fun(t: any, k, v)            @ Intercepts key assignment (`t[k] = v`). Use `rawset(t, k, v)` to skip this metamethod.
--- @field __pow? fun(a, b): any                    @ Exponentiation operator (`a ^ b`)
--- @field __sub? fun(a, b): any                    @ Subtraction operator (`a - b`)
--- @field __tostring? fun(t: any): string          @ Controls what is returned by `tostring(t)`.
--- @field __unm? fun(a): any                       @ Negation operator (`-a`)

--- @param object table
--- @return metatable?
function getmetatable(object) end

--- Represents a custom ring pattern that can be assigned to an agent.
--- @class RingPattern

--- Indicates whether this engine build has Developer Mode enabled.
--- @type boolean
DEVMODE = nil

--- Gets the number of seconds elapsed since the engine was initialized.
--- @return number
function engine_time() end

--- Gets the number of seconds elapsed since the current call started.
--- Returns 0 if no call is active.
--- @return number
function call_time() end

--- @param agent_id integer
--- @param loaded boolean
function set_agent_sounds_loaded(agent_id, loaded) end

--- @class PerlinNoise
C_PerlinNoise = {}

--- Creates a new Perlin noise sampler with the specified parameters.
--- @param octaves integer @ The number of octaves (layers) to add to the noise.
--- @param frequency number @ The number of noise cycles per unit length.
--- @param persistence number @ The amplitude multiplier for each successive octave.
--- @param lacunarity number @ The frequency multiplier for each successive octave.
--- @param seed? integer @ (Optional) The 32-bit seed of the noise generator. If not specified, uses a random seed generated by the global RNG.
--- @return PerlinNoise
function PerlinNoise(octaves, frequency, persistence, lacunarity, seed) end

--- Gets the noise value at the specified coordinates.
--- @param x number @ The X coordinate to sample at.
--- @param y number @ The Y coordinate to sample at.
--- @return number
function C_PerlinNoise:sample(x, y) end