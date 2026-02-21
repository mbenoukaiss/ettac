use std::fmt::{Display, Formatter};
use mlua::prelude::LuaError;

#[derive(Debug)]
pub enum Error {
    LuaError(LuaError),
    SetupError(String),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::LuaError(err) => write!(f, "Lua error: {}", err),
            Error::SetupError(err) => write!(f, "Setup error: {}", err),
        }
    }
}

impl From<mlua::Error> for Error {
    fn from(value: LuaError) -> Self {
        Self::LuaError(value)
    }
}
