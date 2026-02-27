use std::fmt::{Display, Formatter};
use mlua::prelude::LuaError;

#[derive(Debug)]
pub enum Error {
    ConfigError(String),
    LuaError(LuaError),
    SetupError(String),
    SshError(String),
    IoError(std::io::Error),
}

impl Error {
    pub fn config(err: impl Into<String>) -> Self {
        Self::ConfigError(err.into())
    }

    pub fn ssh(err: impl Into<String>) -> Self {
        Self::SshError(err.into())
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ConfigError(err) => write!(f, "Configuration error: {}", err),
            Error::LuaError(err) => write!(f, "Lua error: {}", err),
            Error::SetupError(err) => write!(f, "Setup error: {}", err),
            Error::SshError(err) => write!(f, "SSH error: {}", err),
            Error::IoError(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl From<mlua::Error> for Error {
    fn from(value: LuaError) -> Self {
        Self::LuaError(value)
    }
}

impl From<ssh2::Error> for Error {
    fn from(value: ssh2::Error) -> Self {
        Self::SshError(value.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}
