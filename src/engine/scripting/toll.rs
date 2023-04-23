use crate::engine::*;

impl<'lua> CursedEngine<'lua> {    
    pub(super) fn load_lua_toll_lib(&'static self) -> LuaResult<()> { 
        let lua = &self.lua;
        let globals = &lua.globals();

        let tbl_toll = lua.create_table().unwrap();

        // toll.is_time_low()
        tbl_toll.set("is_time_low", lua.create_function(move |_, ()| {
            Ok(self.is_time_credit_low())
        })?)?;

        // toll.time_left()
        tbl_toll.set("time_left", lua.create_function(move |_, ()| {
            Ok(self.remaining_time_credit().map_or( f64::INFINITY, |d| d.as_secs_f64()))
        })?)?;

        // toll.current_call_rate()
        tbl_toll.set("current_call_rate", lua.create_function(move |_, ()| {
            Ok(self.current_call_rate())
        })?)?;

        // toll.is_current_call_free()
        tbl_toll.set("is_current_call_free", lua.create_function(move |_, ()| {
            Ok(self.is_current_call_free())
        })?)?;

        // toll.is_awaiting_deposit()
        tbl_toll.set("is_awaiting_deposit", lua.create_function(move |_, ()| {
            Ok(self.awaiting_initial_deposit())
        })?)?;

        globals.set("toll", tbl_toll)?;
        
        Ok(())
    }
}