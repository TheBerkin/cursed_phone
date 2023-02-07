--- Represents a cron schedule that can be referenced in a job system.
--- @class CronSchedule

if not cron then
    cron = {}
    
    --- Creates a new cron schedule from the specified expression.
    --- @param expr string @ The cron expression to use for the schedule.
    --- @return CronSchedule?
    --- @diagnostic disable-next-line: missing-return
    function cron.create(expr) end


    --- Advances the internal clock of the specified cron schedule to the current time. 
    --- This function returns two boolean values: the first value indicates whether the schedule has jobs remaining (present and future),
    --- and the second value indicates whether a job has passed since the last tick.
    --- @param schedule CronSchedule
    --- @return boolean, boolean
    --- @diagnostic disable-next-line: missing-return
    function cron.tick(schedule) end
end

--- @async
--- Asynchronously runs the specified job until the schedule runs out of jobs.
--- @param cron_expr string @ The cron expression for the schedule.
--- @param job async fun() @ The job to run.
function cron.run_job(cron_expr, job)
    assert(type(cron_expr) == 'string', "cron schedule expression must be a string")
    local schedule = cron.create(cron_expr)
    if not schedule then error("invalid cron schedule expression: '" .. cron_expr .. "'") end
    local has_jobs = true
    local job_triggered = false
    repeat
        has_jobs, job_triggered = cron.tick(schedule)
        if job_triggered then
            job()
        end
        agent.intent(AGENT_INTENT_YIELD)
    until not has_jobs
end