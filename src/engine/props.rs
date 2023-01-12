use mlua::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AgentRole {
    Normal = 0,
    Intercept = 1,
    Tollmaster = 2
}

const ALL_AGENT_ROLES: &[AgentRole] = { use AgentRole::*; &[Normal, Intercept, Tollmaster] };

impl From<usize> for AgentRole {
    fn from(value: usize) -> AgentRole {
        ALL_AGENT_ROLES[value]
    }
}

#[derive(Copy, Clone, Debug)]
pub enum AgentState {
    /// Agent is currently idle.
    Idle = 0,
    /// Agent is placing a call.
    OutgoingCall = 1,
    /// Agent is receiving a call.
    IncomingCall = 2,
    /// Agent is in a call.
    Call = 3
}

const ALL_AGENT_STATES: &[AgentState] = { use AgentState::*; &[Idle, OutgoingCall, IncomingCall, Call] };

impl From<usize> for AgentState {
    fn from(value: usize) -> AgentState {
        ALL_AGENT_STATES[value]
    }
}

impl AgentState {
    pub fn as_index(self) -> usize {
        self as usize
    }
}

/// Provides reason codes to pass to an agent when connects to a call.
pub enum CallReason {
    /// No call reason given.
    None = 0,
    /// Call was placed because of an off-hook timeout.
    OffHook = 1,
    /// Call was placed because the originally dialed number was disconnected.
    NumberDisconnected = 2,
    /// Call was placed by the user.
    UserInit = 3,
    /// Call was placed by an agent.
    AgentInit = 4
}

impl From<usize> for CallReason {
    fn from(value: usize) -> Self {
        use CallReason::*;
        match value {
            0 => None,
            1 => OffHook,
            2 => NumberDisconnected,
            3 => UserInit,
            4 => AgentInit,
            _ => None
        }
    }
}

impl CallReason {
    pub fn as_index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Debug)]
pub enum AgentIntent {
    Idle,
    AcceptCall,
    EndCall,
    CallUser,
    Wait,
    ReadDigit,
    ForwardCall(String),
    StateEnded(AgentState)
}

impl AgentIntent {
    pub fn from_lua_value(status_code: i32, status_data: LuaValue) -> AgentIntent {
        match status_code {
            0 => AgentIntent::Idle,
            1 => AgentIntent::AcceptCall,
            2 => AgentIntent::EndCall,
            3 => AgentIntent::CallUser,
            4 => AgentIntent::Wait,
            5 => AgentIntent::ReadDigit,
            6 => match status_data {
                LuaValue::String(s) => AgentIntent::ForwardCall(String::from(s.to_str().unwrap())),
                _ => AgentIntent::ForwardCall(String::from("A"))
            },
            7 => match status_data {
                LuaValue::Integer(n) => AgentIntent::StateEnded(AgentState::from(n as usize)),
                _ => AgentIntent::Idle
            },
            _ => AgentIntent::Idle
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum AgentData {
    None,
    Digit(char),
    LineBusy
}

impl AgentData {
    pub fn to_code(&self) -> usize {
        match self {
            AgentData::None => 0,
            AgentData::Digit(_) => 1,
            AgentData::LineBusy => 2
        }
    }
}