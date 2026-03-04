use crate::Error;
use crate::error::SetupError;
use partially::Partial;
use std::fmt::Debug;
use std::rc::Rc;
use crate::access::Access;

#[derive(Debug)]
pub struct Context {
    pub host: Host,
    pub access: Access,
}

#[derive(Partial, Debug)]
#[partially(derive(Default, Clone, Debug))]
pub struct Host {
    pub recipe: Rc<dyn Callable>,
    pub repository: String,
    pub keep_releases: i8,
    pub persistent_files: Vec<String>,
    pub persistent_dirs: Vec<String>,
    pub labels: Vec<String>,

    #[partially(as_type = "Option<PartialSshCredentials>")]
    pub ssh: Option<SshCredentials>,
    pub path: String,
}

impl TryFrom<PartialHost> for Host {
    type Error = SetupError;

    fn try_from(value: PartialHost) -> Result<Self, Self::Error> {
        Ok(Host {
            recipe: value.recipe.ok_or(SetupError::MissingRecipe)?,
            repository: value.repository.ok_or(SetupError::MissingRepository)?,
            keep_releases: value.keep_releases.unwrap_or_default(),
            persistent_files: value.persistent_files.unwrap_or_default(),
            persistent_dirs: value.persistent_dirs.unwrap_or_default(),
            labels: value.labels.unwrap_or_default(),
            ssh: value.ssh.map(SshCredentials::try_from).transpose()?,
            path: value.path.ok_or(SetupError::MissingPath)?,
        })
    }
}

#[derive(Partial, Debug)]
#[partially(derive(Clone, Default, Debug))]
pub struct SshCredentials {
    pub hostname: String,
    pub port: u16,
    pub user: String,
    pub credential: AuthMethod,
}

impl PartialSshCredentials {
    pub fn is_empty(&self) -> bool {
        self.hostname.is_none()
            && self.port.is_none()
            && self.user.is_none()
            && self.credential.is_none()
    }
}

//TODO: check if really necessary
impl From<PartialSshCredentials> for Option<SshCredentials> {
    fn from(partial: PartialSshCredentials) -> Self {
        SshCredentials::try_from(partial).ok()
    }
}

impl TryFrom<PartialSshCredentials> for SshCredentials {
    type Error = SetupError;

    fn try_from(value: PartialSshCredentials) -> Result<Self, Self::Error> {
        let PartialSshCredentials {
            hostname,
            port,
            user,
            credential,
        } = value;

        match (hostname, user, credential) {
            (Some(hostname), Some(user), Some(credential)) => Ok(SshCredentials {
                hostname,
                port: port.unwrap_or(22),
                user,
                credential,
            }),
            (hostname, user, credential) => {
                let mut missing = Vec::with_capacity(3);
                if hostname.is_none() {
                    missing.push("hostname");
                }

                if user.is_none() {
                    missing.push("user");
                }

                if credential.is_none() {
                    missing.push("password or private_key");
                }

                Err(SetupError::MissingCredentials(missing))
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum AuthMethod {
    Password(String),
    Key(String, Option<String>),
}

pub trait Callable: Debug {
    fn call(&self) -> Result<(), Error>;
}
