use crate::access::AccessError;
use mlua::prelude::LuaError;
use std::sync::Arc;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("hosts `{0:?}` not found")]
    UnknownHosts(Vec<String>),
    #[error("invalid deploy config in setup(): {0}")]
    SetupError(#[from] SetupError),
    #[error("script parsing error : {0}")]
    ScriptParsing(String),
    #[error("script runtime error : {0}")]
    ScriptRuntime(LuaError),
    #[error("ssh error : {0}")]
    SshError(#[from] libssh_rs::Error),
    #[error("string `{0}` is not a valid base64 string")]
    InvalidBase64(String),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("unparseable command: {0}")]
    UnparseableCommand(String),
    #[error("access error: {0}")]
    AccessError(#[from] AccessError),
}

#[derive(ThisError, Debug)]
pub enum SetupError {
    #[error("recipe is required")]
    MissingRecipe,
    #[error("repository is required")]
    MissingRepository,
    #[error("path is required")]
    MissingPath,
    #[error("missing ssh credentials {0:?}")]
    MissingCredentials(Vec<&'static str>),
}

impl From<Error> for LuaError {
    fn from(err: Error) -> Self {
        LuaError::ExternalError(Arc::new(err))
    }
}

impl From<LuaError> for Error {
    fn from(err: LuaError) -> Self {
        match err {
            LuaError::SyntaxError { message, .. } => Error::ScriptParsing(message),
            err => Error::ScriptRuntime(err),
        }
    }
}
