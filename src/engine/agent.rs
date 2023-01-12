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
    func_tick: LuaFunction<'lua>
}

impl<'lua> AgentModule<'lua> {
    pub fn from_file(lua: &'lua Lua, path: &Path) -> Result<Self, String> {
        let src = fs::read_to_string(path).expect("Unable to read Lua source file");
        let module_chunk = lua.load(&src).set_name(path.to_str().unwrap()).unwrap();
        let module = module_chunk.eval::<LuaTable>();
        match module {
            Ok(table) => {
                let name = table.raw_get("_name").expect(format!("Agent module '{:?}' requires a name", path).as_ref());
                let phone_number = table.raw_get("_phone_number").unwrap();
                let role = AgentRole::from(table.raw_get::<&'static str, usize>("_role").unwrap());
                let ringback_enabled: bool = table.raw_get("_ringback_enabled").unwrap_or(true);
                let func_load: Option<LuaFunction<'lua>> = table.raw_get("load").unwrap();
                let func_unload = table.raw_get("unload").unwrap();
                let func_tick = table.get("tick").expect("tick() function not found");
                let mut required_sound_banks: Vec<String> = Default::default();
                let mut custom_price = None;

                if let Ok(true) = table.raw_get("_has_custom_price") {
                    if let Ok(price) = table.raw_get("_custom_price") {
                        custom_price = price;
                    }
                }

                // Start state machine
                table.call_method::<&str, _, ()>("start", ()).expect(format!("Unable to start state machine for {}", name).as_str());

                // Call load() if available
                if let Some(func_load) = &func_load {
                    let load_args = lua.create_table().unwrap();
                    load_args.set("path", path.to_str()).unwrap();
                    if let Err(err) = func_load.call::<LuaTable, ()>(load_args) {
                        return Err(format!("Error while calling agent loader: {:#?}", err));
                    }
                }      
                
                // Get required sound banks
                if let Ok(bank_name_table) = table.raw_get::<&'static str, LuaTable>("_required_sound_banks") {
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
                    tbl_module: table,
                    name,
                    role,
                    phone_number,
                    custom_price,
                    func_load,
                    func_unload,
                    func_tick,
                })
            },
            Err(err) => Err(format!("Unable to load agent module: {:#?}", err))
        }
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

    pub fn register_id(&self, id: AgentId) {
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
        self.tbl_module.get("_is_suspended").unwrap_or(false)
    }

    pub fn set_reason(&self, reason: CallReason) -> LuaResult<()> {
        self.tbl_module.call_method("set_reason", reason.as_index())?;
        Ok(())
    }

    pub fn state(&self) -> LuaResult<AgentState> {
        let raw_state = self.tbl_module.get::<&str, usize>("_state")?;
        Ok(AgentState::from(raw_state))
    }

    #[inline]
    pub fn tick(&self, data: AgentData) -> LuaResult<AgentIntent> {
        if self.suspended() {
            return Ok(AgentIntent::Idle)
        }

        let agent_table = self.tbl_module.clone();
        let data_code = data.to_code();

        // Tick agent
        let (intent_code, intent_data) = match data {
            AgentData::None => self.func_tick.call((agent_table, data_code))?,
            AgentData::Digit(digit) => self.func_tick.call((agent_table, data_code, digit.to_string()))?,
            AgentData::LineBusy => self.func_tick.call((agent_table, data_code))?
        };

        let intent = AgentIntent::from_lua_value(intent_code, intent_data);
        Ok(intent)
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