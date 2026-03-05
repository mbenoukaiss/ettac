use crate::access::AccessError;
use mlua::prelude::LuaError;
use thiserror::Error as ThisError;

#[macro_export]
macro_rules! impl_error_try {
    ($error_ty:ty) => {
        impl std::ops::FromResidual<$error_ty> for $error_ty {
            fn from_residual(residual: $error_ty) -> Self {
                residual
            }
        }

        impl std::ops::FromResidual<Result<std::convert::Infallible, $error_ty>> for $error_ty {
            fn from_residual(residual: Result<std::convert::Infallible, $error_ty>) -> Self {
                match residual {
                    Ok(never) => match never {},
                    Err(err) => err,
                }
            }
        }

        impl std::ops::Try for $error_ty {
            type Output = ();
            type Residual = Result<std::convert::Infallible, $error_ty>;

            fn from_output(_: Self::Output) -> Self {
                panic!("from_output can not be called on error type");
            }

            fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
                std::ops::ControlFlow::Break(Err(self))
            }
        }
    };
}

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("no script found at path `{0:?}`")]
    ScriptNotFound(std::path::PathBuf),
    #[error("hosts `{0:?}` not found")]
    UnknownHosts(Vec<String>),
    #[error("invalid deploy config in setup(): {0}")]
    Setup(#[from] SetupError),
    #[error("script parsing error : {0}")]
    ScriptParsing(String),
    #[error("script runtime error : {0}")]
    ScriptRuntime(LuaError),
    #[error("ssh error : {0}")]
    Ssh(#[from] libssh_rs::Error),
    #[error("string `{0}` is not a valid base64 string")]
    InvalidBase64(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("unparseable command: {0}")]
    UnparseableCommand(String),
    #[error("access error: {0}")]
    Access(#[from] AccessError),
}

impl_error_try!(Error);

#[derive(ThisError, Debug)]
#[allow(clippy::enum_variant_names)]
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

impl_error_try!(SetupError);

impl From<Error> for LuaError {
    fn from(err: Error) -> Self {
        LuaError::external(err)
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
