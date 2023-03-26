use crate::engine::scripting::random::{LuaRandom, LuaPerlinSampler};

use super::*;
use std::{error::Error, fmt::Display};
use log;
use rand::RngCore;
use logos::{Logos, Lexer};

mod cron;
mod gpio;
mod phone;
mod logging;
mod random;
mod sound;
mod toll;

#[derive(Debug)]
pub(self) struct CustomLuaError {
    message: String
}

impl CustomLuaError {
    pub fn new(message: String) -> Self {
        Self {
            message
        }
    }
}

impl Error for CustomLuaError {}

impl Display for CustomLuaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

macro_rules! lua_error {
    ($($arg:tt)*) => {
        return Err(LuaError::ExternalError(Arc::new(crate::engine::scripting::CustomLuaError::new(format!($($arg)*)))))
    }
}
pub(self) use lua_error;

#[allow(unused_must_use)]
impl<'lua> CursedEngine<'lua> {    
    pub fn load_lua_api(&'static self) -> LuaResult<()> {
        info!("Setting up Lua...");
    
        let lua = &self.lua;

        let globals = &lua.globals();

        globals.set("DEVMODE", cfg!(feature = "devmode"));
        
        // Override print()
        globals.set("print", lua.create_function(move |lua, values: LuaMultiValue| {
            Self::lua_log_print(lua, values, log::Level::Info, 0);
            Ok(())
        })?)?;
        
        // Global engine functions
        
        globals.set("engine_time", lua.create_function(move |_, ()| {
            let run_time = self.start_time.elapsed().as_secs_f64();
            Ok(run_time)
        })?)?;

        globals.set("call_time", lua.create_function(move |_, ()| {
            match self.state() {
                PhoneLineState::Connected => {
                    return Ok(self.current_state_time().as_secs_f64());
                }
                _ => Ok(0.0)
            }
        })?)?;
        
        // set_agent_sounds_loaded(agent_id, loaded)
        globals.set("set_agent_sounds_loaded", lua.create_function(move |_, (agent_id, load): (AgentId, bool)| {
            let sound_engine = &self.sound_engine;
            if let Some(agent) = self.lookup_agent_id(agent_id) {
                if load {
                    agent.load_sound_banks(sound_engine)
                } else {
                    agent.unload_sound_banks(sound_engine)
                }
            }
            Ok(())
        })?)?;
        
        globals.set("Rng", lua.create_function(move |_, seed: Option<u64>| {
            Ok(LuaRandom::with_seed(seed.unwrap_or_else(|| rand::thread_rng().next_u64())))
        })?)?;

        globals.set("PerlinNoise", lua.create_function(move |_, (octaves, frequency, persistence, lacunarity, seed): (i32, f64, f64, f64, Option<i32>)| {
            Ok(LuaPerlinSampler::new(octaves, frequency, persistence, lacunarity, seed.unwrap_or_else(|| rand::thread_rng().gen())))
        })?)?;
        
        // ====================================================
        // ============= OTHER ENGINE LIBRARIES ===============
        // ====================================================

        self.load_lua_cron_lib()?;
        self.load_lua_gpio_lib()?;
        self.load_lua_phone_lib()?;
        self.load_lua_sound_lib()?;
        self.load_lua_toll_lib()?;
        self.load_lua_log_lib()?;

        // Run API scripts
        self.run_scripts_in_path(self.scripts_root.clone())?;
    
        Ok(())
    }

    fn lua_log_print(lua: &Lua, values: LuaMultiValue, level: log::Level, stack_level: usize) -> LuaResult<()> {
        let mut buffer = String::new();
        
        if let Some(debug_info) = lua.inspect_stack(stack_level.saturating_add(1)) {
            let src_name = debug_info.source().source.map(String::from_utf8_lossy).unwrap_or_default();
            let msg = format!("[{}:{}] ", src_name, debug_info.curr_line());
            buffer.push_str(msg.as_str());
        }

        for (i, val) in values.iter().enumerate() {
            if i > 0 {
                buffer.push('\t');
            }

            if let Some(val_str) = lua.coerce_string(val.clone())? {
                if let Ok(s) = val_str.to_str() {
                    let expanded = expand_ansi_format_string(s);
                    buffer.push_str(expanded.as_str());
                }
            }
        }

        log::log!(level, "{}", buffer);
        Ok(())
    }
}

#[derive(Logos, Debug, PartialEq)]
enum AnsiStringToken {
    #[regex(r"\[:[a-zA-Z0-9]*\]", priority = 10, callback = parse_ansi_format_code)]
    Format(String),
    #[token("[::")]
    EscapeFormat,
    #[regex(r"[^\[]+", priority = 1)]
    #[error]
    Text
}

fn parse_ansi_format_code(lex: &mut Lexer<AnsiStringToken>) -> String {
    let slice = lex.slice();
    let slice_len = slice.len();
    slice[2..(slice_len - 1)].to_string()
}

fn expand_ansi_format_string(s: &str) -> String {
    let mut lex = AnsiStringToken::lexer(s);
    let mut output = String::new();
    let mut has_ansi = false;
    let mut ansi_cleared = false;
    while let Some(token) = lex.next() {
        match token {
            AnsiStringToken::Format(fmt) => {
                output.push_str("\x1b[");
                for (i, fmt_alias) in fmt.chars().enumerate() {
                    ansi_cleared = false;

                    if i > 0 {
                        output.push(';');
                    }

                    let fmt_code = match fmt_alias {
                        'z' => {
                            ansi_cleared = true;
                            "0"
                        },
                        'h' => "1",
                        'l' => "2",
                        'n' => "22",
                        'i' => "3",
                        'u' => "4",
                        'x' => "9",
                        'k' => "30",
                        'r' => "31",
                        'g' => "32",
                        'y' => "33",
                        'b' => "34",
                        'm' => "35",
                        'c' => "36",
                        'w' => "37",
                        'd' => "39",
                        'K' => "40",
                        'R' => "41",
                        'G' => "42",
                        'Y' => "43",
                        'B' => "44",
                        'M' => "45",
                        'C' => "46",
                        'W' => "47",
                        'D' => "49",
                        _ => continue    
                    };
                    
                    output.push_str(fmt_code);
                    has_ansi = true;
                }
                output.push_str("m");
            },
            AnsiStringToken::EscapeFormat => output.push_str("[:"),
            AnsiStringToken::Text => output.push_str(lex.slice()),
        }
    }

    if has_ansi && !ansi_cleared {
        // Clear formatting at end of string
        output.push_str("\x1b[m");
    }

    output
}