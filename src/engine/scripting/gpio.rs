use crate::engine::*;

#[cfg(feature = "rpi")]
use crate::gpio::*;

impl<'lua> CursedEngine<'lua> {    
    pub(super) fn load_lua_gpio_lib(&'static self) -> LuaResult<()> { 
        #[cfg(feature = "rpi")]
        {
            let lua = &self.lua;
            let globals = &lua.globals();

            let tbl_gpio = lua.create_table()?;

            tbl_gpio.set("register_input", lua.create_function(move |_, (pin, pull, bounce_time): (u8, Option<String>, Option<f64>)| {
                Ok(self.gpio.borrow_mut().register_input(
                    pin, 
                    pull.map_or(Pull::None, |v| Pull::from(&v)), 
                    bounce_time.map(|t| Duration::from_secs_f64(t))
                ).to_lua_err()?)
            })?)?;

            tbl_gpio.set("register_output", lua.create_function(move |_, pin: u8| {
                Ok(self.gpio.borrow_mut().register_output(pin).to_lua_err()?)
            })?)?;

            tbl_gpio.set("read_pin", lua.create_function(move |_, pin: u8| {
                Ok(self.gpio.borrow().read_pin(pin))
            })?)?;

            tbl_gpio.set("write_pin", lua.create_function(move |_, (pin, logic_level): (u8, bool)| {
                Ok(self.gpio.borrow_mut().write_pin(pin, logic_level))
            })?)?;

            tbl_gpio.set("set_pwm", lua.create_function(move |_, (pin, period, pulse_width): (u8, f64, f64)| {
                Ok(self.gpio.borrow_mut().set_pwm(pin, period, pulse_width).to_lua_err()?)
            })?)?;

            tbl_gpio.set("clear_pwm", lua.create_function(move |_, pin: u8| {
                Ok(self.gpio.borrow_mut().clear_pwm(pin).to_lua_err()?)
            })?)?;

            tbl_gpio.set("unregister", lua.create_function(move |_, pin: u8| {
                Ok(self.gpio.borrow_mut().unregister(pin))
            })?)?;

            tbl_gpio.set("unregister_all", lua.create_function(move |_, ()| {
                Ok(self.gpio.borrow_mut().unregister_all())
            })?)?;

            globals.set("gpio", tbl_gpio)?;
        }
        Ok(())
    }
}