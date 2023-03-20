use crate::engine::*;

impl<'lua> CursedEngine<'lua> {    
    pub(super) fn load_lua_log_lib(&'static self) -> LuaResult<()> { 
        let lua = &self.lua;
        let globals = &lua.globals();

        let tbl_log = lua.create_table().unwrap();

        tbl_log.set("info", lua.create_function(move |lua, args: LuaMultiValue| {
            Self::lua_log_print(lua, args, log::Level::Info, 0)
        })?)?;

        tbl_log.set("info_caller", lua.create_function(move |lua, (level, args): (usize, LuaMultiValue)| {
            Self::lua_log_print(lua, args, log::Level::Info, level)
        })?)?;

        tbl_log.set("warn", lua.create_function(move |lua, args: LuaMultiValue| {
            Self::lua_log_print(lua, args, log::Level::Warn, 0)
        })?)?;

        tbl_log.set("warn_caller", lua.create_function(move |lua, (level, args): (usize, LuaMultiValue)| {
            Self::lua_log_print(lua, args, log::Level::Warn, level)
        })?)?;

        tbl_log.set("error", lua.create_function(move |lua, args: LuaMultiValue| {
            Self::lua_log_print(lua, args, log::Level::Error, 0)
        })?)?;

        tbl_log.set("error_caller", lua.create_function(move |lua, (level, args): (usize, LuaMultiValue)| {
            Self::lua_log_print(lua, args, log::Level::Error, level)
        })?)?;

        globals.set("log", tbl_log)?;
        
        Ok(())
    }
}