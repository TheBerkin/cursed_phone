--- Represents a cron schedule that can be referenced in a job system.
--- @class CronSchedule

if not cron then
    cron = {}
    
    --- Creates a new cron schedule from the specified expression.
    --- @param expr string @ The cron expression to use for the schedule.
    --- @return CronSchedule
    --- @diagnostic disable-next-line: missing-return
    function cron.create(expr) end


    --- Advances the internal clock of the specified cron schedule to the current time. 
    --- If the schedule has reached its next job since the last tick, the function returns `true`; otherwise, `false`.
    --- @param schedule CronSchedule
    --- @diagnostic disable-next-line: missing-return
    function cron.tick(schedule) end
end


