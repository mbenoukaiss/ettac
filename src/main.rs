mod config;
pub mod library;
mod runners;
mod error;
mod context;

use std::rc::Rc;
pub use error::Error;

use crate::config::Config;
use crate::context::Context;
use crate::runners::{LuaRunner, Runner};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = argh::from_env::<Config>();
    let config = Box::leak(Box::new(config));

    let mut runner = LuaRunner::new(config);
    runner.init()?;

    let mut hosts = runner.get_hosts()?;
    println!("{:#?}", hosts);

    let context = Context {
        host: hosts.remove("prod").unwrap(),
    };

    runner.run(Rc::new(context))?;

    Ok(())
}
