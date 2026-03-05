mod lua;

pub use lua::LuaRunner;
use std::collections::HashMap;

use crate::Error;
use crate::context::{Context, Host};

pub trait Runner {
    fn init(&mut self) -> Result<(), Error>;
    fn get_hosts(&mut self) -> Result<HashMap<String, Host>, Error>;
    fn run(&mut self, ctx: Context) -> Result<(), Error>;
}
