--- @meta

--- Logging functions with ANSI formatting shorthand support.
--- 
--- ## ANSI Formatting Shorthand
---
--- To insert ANSI formatting codes use the notation `[:___]` where `___` is some string of formatting characters.
--- The following format characters are supported:
--- * `z` &mdash; Clear all formatting
--- * `h` &mdash; High intensity (bright / bold)
--- * `l` &mdash; Low intensity (dim / unbold)
--- * `n` &mdash; Normal intensity
--- * `i` &mdash; Italic
--- * `u` &mdash; Underline
--- * `x` &mdash; Strikethrough
--- * `k` &mdash; Black (foreground)
--- * `r` &mdash; Red (foreground)
--- * `g` &mdash; Green (foreground)
--- * `y` &mdash; Yellow (foreground)
--- * `b` &mdash; Blue (foreground)
--- * `m` &mdash; Magenta (foreground)
--- * `c` &mdash; Cyan (foreground)
--- * `w` &mdash; White (foreground)
--- * `d` &mdash; Default foreground color
--- * `K` &mdash; Black (background)
--- * `R` &mdash; Red (background)
--- * `G` &mdash; Green (background)
--- * `Y` &mdash; Yellow (background)
--- * `B` &mdash; Blue (background)
--- * `M` &mdash; Magenta (background)
--- * `C` &mdash; Cyan (background)
--- * `W` &mdash; White (background)
--- * `D` &mdash; Default background color
--- @class LogLib
log = {}

--- Prints an informational log message with the current script path and line number.
--- See 'log' documentation for ANSI shorthands.
function log.info(...) end

--- Prints an informational log message with the current script path and line number.
--- @param level integer
function log.info_caller(level, ...) end

--- Prints a warning log message with the current script path and line number.
function log.warn(...) end

--- Prints a warning log message with the current script path and line number.
--- @param level integer
function log.warn_caller(level, ...) end

--- Prints an error log message with the current script path and line number.
function log.error(...) end

--- Prints an error log message with the current script path and line number.
--- @param level integer
function log.error_caller(level, ...) end