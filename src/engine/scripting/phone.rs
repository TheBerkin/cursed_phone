use crate::engine::*;

impl<'lua> CursedEngine<'lua> {    
    pub(super) fn load_lua_phone_lib(&'static self) -> LuaResult<()> { 
        let lua = &self.lua;
        let globals = &lua.globals();

        let tbl_phone = lua.create_table()?;

        tbl_phone.set("last_caller_id", lua.create_function(move |_, ()| {
            Ok(self.last_caller_id.get())
        })?)?;

        tbl_phone.set("last_dialed_number", lua.create_function(move |_, ()| {
            Ok(self.last_dialed_number.borrow().clone())
        })?)?;

        tbl_phone.set("dial", lua.create_function(move |_, digits: String| {
            for digit in digits.chars() {
                self.handle_host_digit(digit);
            }
            Ok(())
        })?)?;

        tbl_phone.set("is_rotary", lua.create_function(move |_, ()| {
            Ok(self.config.rotary.enabled)
        })?)?;

        tbl_phone.set("is_rotary_dial_resting", lua.create_function(move |_, ()| {
            Ok(if self.config.rotary.enabled {
                Some(self.rotary_resting.get())
            } else {
                None
            })
        })?)?;

        tbl_phone.set("is_on_hook", lua.create_function(move |_, ()| {
            Ok(self.switchhook_closed.get())
        })?)?;

        tbl_phone.set("ring", lua.create_function(move |_, pattern: LuaValue| {
            let pattern = match pattern {
                LuaValue::String(expr) => match RingPattern::try_parse(expr.to_str().to_lua_err()?) {
                    Some(pattern) => Arc::new(pattern),
                    None => return Err(LuaError::RuntimeError(format!("invalid ring pattern expression: '{}'", expr.to_string_lossy())))
                },
                LuaValue::UserData(userdata) => userdata.clone().take::<LuaRingPattern>()?.0,
                other => return Err(LuaError::RuntimeError(format!("cannot use type '{}' as ring pattern", other.type_name())))
            };
            self.send_output(PhoneOutputSignal::Ring(Some(pattern)));
            Ok(())
        })?)?;

        tbl_phone.set("stop_ringing", lua.create_function(move |_, ()| {
            self.send_output(PhoneOutputSignal::Ring(None));
            Ok(())
        })?)?;

        tbl_phone.set("compile_ring_pattern", lua.create_function(move |_, expr: String| {
            if let Some(pattern) = RingPattern::try_parse(expr.as_str()) {
                Ok((true, Some(LuaRingPattern(Arc::new(pattern)))))
            } else {
                Ok((false, None))
            }
        })?)?;

        globals.set("call_dialed_number", lua.create_function(move |_, ()| {
            return Ok(self.called_number.borrow().clone())
        })?)?;

        globals.set("phone", tbl_phone)?;

        Ok(())
    }
}