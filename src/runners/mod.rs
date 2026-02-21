mod lua;

pub use lua::LuaRunner;

use crate::Error;
use std::collections::HashMap;

pub trait Runner {
    fn init(&mut self) -> Result<(), Error>;
    fn get_hosts(&mut self) -> Result<Hosts, Error>;
}

pub type Hosts = HashMap<String, Host>;
pub type Host = HashMap<String, Unknown>;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Unknown {
    None,
    Int(i32),
    Boolean(bool),
    Float(f32),
    String(String),
    Vec(Vec<Unknown>),
    Map(HashMap<String, Unknown>),
}
