--- @meta

--- @class LogLib
log = {}

--- Prints an informational log message with the current script path and line number.
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