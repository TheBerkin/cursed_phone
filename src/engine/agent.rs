use std::cell::Cell;
use std::error::Error;

use super::*;

pub struct AgentModule<'lua> {
    id: RefCell<Option<AgentId>>,
    name: String,
    phone_number: Option<String>,
    role: AgentRole,
    custom_price: Option<u32>,
    ringback_enabled: bool,
    required_sound_banks: Vec<String>,
    tbl_module: LuaTable<'lua>,
    func_load: Option<LuaFunction<'lua>>,
    func_unload: Option<LuaFunction<'lua>>,
    func_tick: LuaFunction<'lua>,
    suspended: Cell<bool>,
}

impl<'lua> AgentModule<'lua> {
    pub fn from_file(lua: &'lua Lua, path: &VfsPath) -> Result<Self, Box<dyn Error>> {
        let src = path.read_to_string()?;
        let module_chunk = lua.load(&src).set_name(path.as_str())?;
        let module = module_chunk.eval::<LuaTable>()?;
        let name = module.raw_get("_name")?;
        let phone_number = module.raw_get("_phone_number")?;
        let role = AgentRole::from(module.raw_get::<&'static str, usize>("_role")?);
        let ringback_enabled: bool = module.raw_get("_ringback_enabled").unwrap_or(true);
        let func_load: Option<LuaFunction<'lua>> = module.raw_get("_on_load")?;
        let func_unload = module.raw_get("_on_unload")?;
        let func_tick = module.get("tick")?;
        let mut required_sound_banks: Vec<String> = Default::default();
        let mut custom_price = None;

        if let Ok(true) = module.raw_get("_has_custom_price") {
            if let Ok(price) = module.raw_get("_custom_price") {
                custom_price = price;
            }
        }   
        
        // Get required sound banks
        if let Ok(bank_name_table) = module.raw_get::<&'static str, LuaTable>("_required_sound_banks") {
            let pairs = bank_name_table.pairs::<String, bool>();
            for pair in pairs {
                if let Ok((bank_name, required)) = pair {
                    if !required || bank_name.is_empty() { continue }
                    required_sound_banks.push(bank_name);
                }
            }
        }

        Ok(Self {
            id: Default::default(),
            required_sound_banks,
            ringback_enabled,
            tbl_module: module,
            name,
            role,
            phone_number,
            custom_price,
            func_load,
            func_unload,
            func_tick,
            suspended: Default::default(),
        })
    }

    pub fn custom_ring_pattern(&self) -> Option<Arc<RingPattern>> {
        self.tbl_module.get::<&str, Option<LuaRingPattern>>("_custom_ring_pattern").ok().flatten().map(|p| p.0)
    }

    pub fn load_sound_banks(&self, sound_engine: &Rc<RefCell<SoundEngine>>) {
        let mut sound_engine = sound_engine.borrow_mut();
        for bank_name in &self.required_sound_banks {
            sound_engine.add_sound_bank_user(bank_name, SoundBankUser(self.id().unwrap()));
        }
    }

    pub fn unload_sound_banks(&self, sound_engine: &Rc<RefCell<SoundEngine>>) {
        let mut sound_engine = sound_engine.borrow_mut();
        for bank_name in &self.required_sound_banks {
            sound_engine.remove_sound_bank_user(bank_name, SoundBankUser(self.id().unwrap()), true);
        }
    }

    pub fn start_state_machine(&self) -> Result<bool, LuaError> {
        self.tbl_module.call_method::<&str, _, bool>("start", ())
    }

    pub fn call_load_handler(&self) -> Result<(), LuaError> {
        if let Some(func_load) = &self.func_load {
            func_load.call::<LuaTable, ()>(self.tbl_module.clone())?;
        }
        Ok(())
    }

    pub fn call_unload_handler(&self) -> Result<(), LuaError> {
        if let Some(func_unload) = &self.func_unload {
            func_unload.call::<LuaTable, ()>(self.tbl_module.clone())?;
        }
        Ok(())
    }

    pub fn register_id(&self, id: AgentId) {
        self.tbl_module.raw_set("_id", id).unwrap();
        self.id.replace(Some(id));
    }

    pub fn id(&self) -> Option<AgentId> {
        *self.id.borrow()
    }

    pub fn role(&self) -> AgentRole {
        self.role
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn ringback_enabled(&self) -> bool {
        self.ringback_enabled
    }

    pub fn phone_number(&self) -> Option<String> {
        self.phone_number.clone()
    }

    pub fn custom_price(&self) -> Option<u32> {
        self.custom_price
    }

    pub fn suspended(&self) -> bool {
        self.suspended.get()
    }

    pub fn set_suspended(&self, suspended: bool) {
        self.suspended.set(suspended)
    }

    pub fn set_call_reason(&self, reason: CallReason) -> LuaResult<()> {
        self.tbl_module.call_method("set_call_reason", reason.as_index())?;
        Ok(())
    }

    pub fn state(&self) -> LuaResult<AgentState> {
        let raw_state = self.tbl_module.get::<&str, usize>("_state")?;
        Ok(AgentState::from(raw_state))
    }

    #[inline]
    pub fn tick(&self, data: AgentIntentResponse) -> LuaResult<(AgentIntent, AgentContinuation)> {
        if self.suspended() {
            return Ok((AgentIntent::Yield, AgentContinuation::NextAgent))
        }

        let agent_table = self.tbl_module.clone();
        let data_code = data.to_code();

        // Tick agent
        let (intent_code, intent_data, should_continue) = match data {
            AgentIntentResponse::None => self.func_tick.call((agent_table, data_code))?,
            AgentIntentResponse::Digit(digit) => self.func_tick.call((agent_table, data_code, digit.to_string()))?,
            AgentIntentResponse::LineBusy => self.func_tick.call((agent_table, data_code))?
        };

        let intent = AgentIntent::from_lua_value(intent_code, intent_data);

        let continuation = if should_continue {
            AgentContinuation::ThisAgent
        } else {
            AgentContinuation::NextAgent
        };

        Ok((intent, continuation))
    }

    pub fn transition_state(&self, state: AgentState) -> LuaResult<()> {
        self.tbl_module.call_method("transition", state.as_index())?;
        Ok(())
    }
}

impl<'lua> Drop for AgentModule<'lua> {
    fn drop(&mut self) {
        if let Some(unload) = &self.func_unload {
            if let Err(error) = unload.call::<(), ()>(()) {
                error!("Agent module '{}' encountered error while unloading: {:?}", self.name, error);
            }
        }
    }
}