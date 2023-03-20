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
        task.intent(IntentCode.YIELD)
    until not has_jobs
end