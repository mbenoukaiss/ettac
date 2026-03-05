use crate::Error;
use crate::context::{AuthMethod, SshCredentials};
use crate::impl_error_try;
use libssh_rs::{Session, SshKey, SshOption};
use std::fmt::Debug;
use std::io::Read;
use std::process::Command;
use thiserror::Error as ThisError;

const UNKNOWN_STATUS: i32 = -42398;

#[derive(ThisError, Debug)]
pub enum AccessError {
    #[error("Directory not found")]
    DirectoryNotFound = 1,
}

impl_error_try!(AccessError);

impl AccessError {
    pub const INTERNAL_ERROR_PREFIX: &'static str = "__ettac_internal_error::";

    pub fn format(error: AccessError) -> String {
        let mut output = String::new();
        output.push_str(Self::INTERNAL_ERROR_PREFIX);
        output.push_str((error as u8).to_string().as_str());

        output
    }

    pub fn is(error: &str) -> bool {
        error.contains(Self::INTERNAL_ERROR_PREFIX)
    }
}

impl From<&String> for AccessError {
    fn from(error: &String) -> Self {
        let prefix = Self::INTERNAL_ERROR_PREFIX;
        let code = error
            .split_at(error.find(prefix).unwrap() + prefix.len())
            .1
            .as_bytes()[0];

        match code {
            1 => AccessError::DirectoryNotFound,
            _ => panic!("unknown access error code: {}", code),
        }
    }
}

pub enum Access {
    Local(String),
    Remote(String, Session),
}

impl Debug for Access {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Access::Local(path) => write!(f, "Access::Local({})", path),
            Access::Remote(path, _) => write!(f, "Access::Remote({})", path),
        }
    }
}

impl Access {
    pub fn run(&self, cmd: &str) -> Result<CommandResult, Error> {
        match self {
            Access::Local(path) => {
                let Some(args) = shlex::split(cmd) else {
                    return Err(Error::UnparseableCommand(cmd.to_string()));
                };

                let output = Command::new(&args[0])
                    .current_dir(path)
                    .args(&args[1..])
                    .output()?;

                Ok(CommandResult {
                    status: output.status.code().unwrap_or(UNKNOWN_STATUS),
                    stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                })
            }
            Access::Remote(path, sess) => {
                let channel = sess.new_channel()?;
                channel.open_session()?;
                channel.request_exec(&format!(
                    "if cd {}; then {}; else >&2 echo '{}'; exit; fi",
                    path,
                    cmd,
                    AccessError::format(AccessError::DirectoryNotFound),
                ))?;

                channel.send_eof()?;

                let mut stdout = String::new();
                channel.stdout().read_to_string(&mut stdout)?;

                let mut stderr = String::new();
                channel.stderr().read_to_string(&mut stderr)?;

                if AccessError::is(&stderr) {
                    AccessError::from(&stderr)?
                }

                Ok(CommandResult {
                    status: channel.get_exit_status().unwrap_or(UNKNOWN_STATUS),
                    stdout,
                    stderr,
                })
            }
        }
    }
}

#[derive(Debug)]
pub struct CommandResult {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

pub fn to(path: impl Into<String>, cred: &Option<SshCredentials>) -> Result<Access, Error> {
    let path = path.into();

    if let Some(cred) = cred {
        let sess = Session::new()?;
        sess.set_option(SshOption::Hostname(cred.hostname.clone()))?;
        sess.set_option(SshOption::Port(cred.port))?;
        sess.set_option(SshOption::User(Some(cred.user.clone())))?;
        sess.options_parse_config(None)?;
        sess.connect()?;

        match &cred.credential {
            AuthMethod::Password(password) => {
                sess.userauth_password(None, Some(password))?;
            }
            AuthMethod::Key(key, passphrase) => {
                let key = SshKey::from_privkey_base64(key, passphrase.as_deref())?;
                sess.userauth_publickey(None, &key)?;
            }
        };

        Ok(Access::Remote(path, sess))
    } else {
        Ok(Access::Local(path))
    }
}
