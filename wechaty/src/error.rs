use std::{error, fmt};

use wechaty_puppet::PuppetError;

pub enum WechatyError {
    Puppet(PuppetError),
    InvalidOperation(String),
    Maybe(String),
    NotLoggedIn,
    NoPayload,
}

impl fmt::Debug for WechatyError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "WechatyError({})", self)
    }
}

impl fmt::Display for WechatyError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WechatyError::Puppet(e) => write!(fmt, "Puppet error: {}", e),
            WechatyError::InvalidOperation(op) => write!(fmt, "Invalid operation: {}", op),
            WechatyError::Maybe(maybe) => write!(fmt, "An error may have occurred: {}", maybe),
            WechatyError::NotLoggedIn => write!(fmt, "User is not logged in"),
            WechatyError::NoPayload => write!(fmt, "Operation cannot be done because the current entity does not have payload due to an unknown previous issue"),
        }
    }
}

impl From<PuppetError> for WechatyError {
    fn from(e: PuppetError) -> Self {
        WechatyError::Puppet(e)
    }
}

impl error::Error for WechatyError {}
