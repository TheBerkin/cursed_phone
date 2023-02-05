use mlua::prelude::*;

use super::AgentId;

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
    /// Agent performed no action.
    Idle,
    /// Agent wants to accept an incoming call.
    AcceptCall,
    /// Agent wants to end an ongoing call.
    EndCall,
    /// Agent wants to call the host.
    CallUser,
    /// Agent is waiting for an operation to complete.
    Wait,
    /// Agent is requesting a digit from the host.
    ReadDigit,
    /// Agent wants to forward the call to a specified phone number.
    ForwardCall(String),
    /// Agent has ended its current state.
    StateEnded(AgentState),
    /// Agent wants to forward the call to a specified Agent ID.
    ForwardCallToId(AgentId),
}

impl AgentIntent {
    pub fn from_lua_value(intent_code: i32, intent_data: LuaValue) -> AgentIntent {
        match intent_code {
            0 => AgentIntent::Idle,
            1 => AgentIntent::AcceptCall,
            2 => AgentIntent::EndCall,
            3 => AgentIntent::CallUser,
            4 => AgentIntent::Wait,
            5 => AgentIntent::ReadDigit,
            6 => match intent_data {
                LuaValue::String(s) => AgentIntent::ForwardCall(String::from(s.to_str().unwrap())),
                _ => AgentIntent::ForwardCall(String::from("A"))
            },
            7 => match intent_data {
                LuaValue::Integer(n) => AgentIntent::StateEnded(AgentState::from(n as usize)),
                _ => AgentIntent::Idle
            },
            8 => match intent_data {
                LuaValue::Integer(n) => AgentIntent::ForwardCallToId(n as usize),
                _ => AgentIntent::Idle
            }
            _ => AgentIntent::Idle
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum AgentIntentResponse {
    None,
    Digit(char),
    LineBusy
}

impl AgentIntentResponse {
    pub fn to_code(&self) -> usize {
        match self {
            AgentIntentResponse::None => 0,
            AgentIntentResponse::Digit(_) => 1,
            AgentIntentResponse::LineBusy => 2
        }
    }
}