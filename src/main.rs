mod config;
pub mod library;
mod runners;
mod error;

pub use error::Error;

use crate::config::Config;
use crate::runners::{LuaRunner, Runner};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = argh::from_env::<Config>();
    let config = Box::leak(Box::new(config));

    let mut runner = LuaRunner::new(config);
    runner.init()?;

    let hosts = runner.get_hosts()?;
    println!("{:#?}", hosts);

    Ok(())
}
