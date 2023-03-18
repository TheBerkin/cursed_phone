--- @meta

--- Represents a cron schedule that can be referenced in a job system.
--- @class CronSchedule

cron = {}
    
--- Creates a new cron schedule from the specified expression.
--- @param expr string @ The cron expression to use for the schedule.
--- @return CronSchedule?
--- @nodiscard
function cron.create(expr) end

--- Advances the internal clock of the specified cron schedule to the current time. 
--- This function returns two boolean values: the first value indicates whether the schedule has jobs remaining (present and future),
--- and the second value indicates whether a job has passed since the last tick.
--- @param schedule CronSchedule
--- @return boolean, boolean
function cron.tick(schedule) end