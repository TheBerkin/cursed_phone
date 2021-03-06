use mlua::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ServiceRole {
    Normal = 0,
    Intercept = 1,
    Tollmaster = 2
}

const ALL_SERVICE_ROLES: &[ServiceRole] = { use ServiceRole::*; &[Normal, Intercept, Tollmaster] };

impl From<usize> for ServiceRole {
    fn from(value: usize) -> ServiceRole {
        ALL_SERVICE_ROLES[value]
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ServiceState {
    /// Service is currently idle.
    Idle = 0,
    /// Service is placing a call.
    OutgoingCall = 1,
    /// Service is receiving a call.
    IncomingCall = 2,
    /// Service is in a call.
    Call = 3
}

const ALL_SERVICE_STATES: &[ServiceState] = { use ServiceState::*; &[Idle, OutgoingCall, IncomingCall, Call] };

impl From<usize> for ServiceState {
    fn from(value: usize) -> ServiceState {
        ALL_SERVICE_STATES[value]
    }
}

impl ServiceState {
    pub fn as_index(self) -> usize {
        self as usize
    }
}

/// Provides reason codes to pass to a service when connects to a call.
pub enum CallReason {
    /// No call reason given.
    None = 0,
    /// Call was placed because of an off-hook timeout.
    OffHook = 1,
    /// Call was placed because the originally dialed number was disconnected.
    NumberDisconnected = 2,
    /// Call was placed by the user.
    UserInit = 3,
    /// Call was placed by a service.
    ServiceInit = 4
}

impl From<usize> for CallReason {
    fn from(value: usize) -> Self {
        use CallReason::*;
        match value {
            0 => None,
            1 => OffHook,
            2 => NumberDisconnected,
            3 => UserInit,
            4 => ServiceInit,
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
pub enum ServiceIntent {
    Idle,
    AcceptCall,
    EndCall,
    CallUser,
    Wait,
    ReadDigit,
    ForwardCall(String),
    StateEnded(ServiceState)
}

impl ServiceIntent {
    pub fn from_lua_value(status_code: i32, status_data: LuaValue) -> ServiceIntent {
        match status_code {
            0 => ServiceIntent::Idle,
            1 => ServiceIntent::AcceptCall,
            2 => ServiceIntent::EndCall,
            3 => ServiceIntent::CallUser,
            4 => ServiceIntent::Wait,
            5 => ServiceIntent::ReadDigit,
            6 => match status_data {
                LuaValue::String(s) => ServiceIntent::ForwardCall(String::from(s.to_str().unwrap())),
                _ => ServiceIntent::ForwardCall(String::from("A"))
            },
            7 => match status_data {
                LuaValue::Integer(n) => ServiceIntent::StateEnded(ServiceState::from(n as usize)),
                _ => ServiceIntent::Idle
            },
            _ => ServiceIntent::Idle
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ServiceData {
    None,
    Digit(char),
    LineBusy
}

impl ServiceData {
    pub fn to_code(&self) -> usize {
        match self {
            ServiceData::None => 0,
            ServiceData::Digit(_) => 1,
            ServiceData::LineBusy => 2
        }
    }
}