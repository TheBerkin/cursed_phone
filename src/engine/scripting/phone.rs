use crate::engine::*;

impl<'lua> CursedEngine<'lua> {    
    pub(super) fn load_lua_phone_lib(&'static self) -> LuaResult<()> { 
        let lua = &self.lua;
        let globals = &lua.globals();

        let tbl_phone = lua.create_table()?;

        // phone.last_caller_id()
        tbl_phone.set("last_caller_id", lua.create_function(move |_, ()| {
            Ok(self.last_caller_id.get())
        })?)?;

        // phone.is_rotary()
        tbl_phone.set("is_rotary", lua.create_function(move |_, ()| {
            Ok(self.host_phone_type == PhoneType::Rotary)
        })?)?;

        // phone.is_rotary_dial_resting()
        tbl_phone.set("is_rotary_dial_resting", lua.create_function(move |_, ()| {
            Ok(if self.host_phone_type == PhoneType::Rotary {
                Some(self.rotary_resting.get())
            } else {
                None
            })
        })?)?;

        // phone.is_on_hook()
        tbl_phone.set("is_on_hook", lua.create_function(move |_, ()| {
            Ok(self.switchhook_closed.get())
        })?)?;

        globals.set("phone", tbl_phone)?;

        Ok(())
    }
}