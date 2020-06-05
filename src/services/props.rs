use mlua::prelude::*;

#[derive(Copy, Clone, Debug)]
pub enum ServiceRole {
    Normal = 0,
    Intercept = 1
}

const ALL_SERVICE_ROLES: &[ServiceRole] = { use ServiceRole::*; &[Normal, Intercept] };

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

#[derive(Clone, Debug)]
pub enum ServiceIntent {
    Idle,
    AcceptCall,
    EndCall,
    CallUser,
    Waiting,
    RequestDigit,
    Forward(String),
    FinishedState(ServiceState)
}

impl ServiceIntent {
    pub fn from_lua_value(status_code: i32, status_data: LuaValue) -> ServiceIntent {
        match status_code {
            0 => ServiceIntent::Idle,
            1 => ServiceIntent::AcceptCall,
            2 => ServiceIntent::EndCall,
            3 => ServiceIntent::CallUser,
            4 => ServiceIntent::Waiting,
            5 => ServiceIntent::RequestDigit,
            6 => match status_data {
                LuaValue::String(s) => ServiceIntent::Forward(String::from(s.to_str().unwrap())),
                _ => ServiceIntent::Forward(String::from("A"))
            },
            7 => match status_data {
                LuaValue::Integer(n) => ServiceIntent::FinishedState(ServiceState::from(n as usize)),
                _ => ServiceIntent::Idle
            },
            _ => ServiceIntent::Idle
        }
    }
}