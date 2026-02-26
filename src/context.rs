use std::fmt::Debug;
use crate::Error;

#[derive(Debug)]
pub struct Context {
    pub host: Host,
}

#[derive(Debug)]
pub struct Host {
    pub recipe: Box<dyn Callable>,
    pub repository: String,
    pub keep_releases: i8,
    pub persistent_files: Vec<String>,
    pub persistent_dirs: Vec<String>,
    pub labels: Vec<String>,

    pub ssh: Option<Ssh>,
    pub path: String,
}

#[derive(Clone, Debug)]
pub struct Ssh {
    pub hostname: String,
    pub port: u16,
    pub user: String,
    pub credential: SshCredential,
}

#[derive(Clone, Debug)]
pub enum SshCredential {
    Password(String),
    PrivateKey(String),
}

pub trait Callable: Debug {
    fn call(&self) -> Result<(), Error>;
}
