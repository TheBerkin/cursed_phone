use std::str::FromStr;

use cron::*;
use chrono::prelude::*;
use crate::engine::*;

#[derive(Clone)]
struct LuaCronSchedule {
    next_time: Rc<Cell<Option<DateTime<Local>>>>,
    cron: Schedule,
}

impl LuaCronSchedule {
    fn new(expr: &str) -> Option<Self> {
        if let Ok(cron) = Schedule::from_str(expr) {
            Some(Self {
                next_time: Rc::new(Cell::new(cron.after_owned(Local::now()).next())),
                cron
            })
        } else {
            None
        }
    }

    fn tick(&self) -> Option<bool> {
        if self.next_time.get().is_none() { return None }
        let now = Local::now();
        if let Some(next_time) = self.next_time.get() {
            if next_time <= now {
                self.next_time.set(self.cron.after(&now).next());
                return Some(true)
            }
        }
        return Some(false)
    }
}

impl LuaUserData for LuaCronSchedule {}

impl<'lua> CursedEngine<'lua> {    
    pub(super) fn load_lua_cron_lib(&'static self) -> LuaResult<()> { 
        let lua = &self.lua;
        let globals = &lua.globals();

        let tbl_cron = lua.create_table()?;

        // cron.create(expr)
        tbl_cron.set("create", lua.create_function(move |_, expr: String| {
            return Ok(LuaCronSchedule::new(expr.as_str()))
        })?)?;

        // cron.tick(schedule)
        tbl_cron.set("tick", lua.create_function(move |_, schedule: LuaCronSchedule| {
            if let Some(triggered) = schedule.tick() {
                return Ok((true, Some(triggered)))
            } else {
                return Ok((false, None))
            }
        })?)?;

        globals.set("cron", tbl_cron)?;
        Ok(())
    }
}