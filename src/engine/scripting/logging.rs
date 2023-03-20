use crate::engine::*;

impl<'lua> CursedEngine<'lua> {    
    pub(super) fn load_lua_log_lib(&'static self) -> LuaResult<()> { 
        let lua = &self.lua;
        let globals = &lua.globals();

        let tbl_log = lua.create_table().unwrap();

        tbl_log.set("info", lua.create_function(move |lua, args: LuaMultiValue| {
            Self::lua_log_print(lua, args, log::Level::Info)
        })?)?;

        tbl_log.set("warn", lua.create_function(move |lua, args: LuaMultiValue| {
            Self::lua_log_print(lua, args, log::Level::Warn)
        })?)?;

        tbl_log.set("error", lua.create_function(move |lua, args: LuaMultiValue| {
            Self::lua_log_print(lua, args, log::Level::Error)
        })?)?;

        globals.set("log", tbl_log)?;
        
        Ok(())
    }
}