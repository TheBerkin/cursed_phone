--- @meta

--- Represents a cron schedule that can be referenced in a job system.
--- @class CronSchedule
local C_CronSchedule = {}

--- Creates a new cron schedule from the specified cron expression. Returns `nil` if the cron expression was invalid.
--- @param expr string @ The cron expression to use for the schedule.
--- @return CronSchedule?
--- @nodiscard
function CronSchedule(expr) end

--- Advances the internal clock of the cron schedule to the current time. 
--- This function returns two boolean values: the first value indicates whether the schedule has jobs remaining (present and future),
--- and the second value indicates whether a job has passed since the last tick.
--- @return boolean, boolean
function C_CronSchedule:tick() end