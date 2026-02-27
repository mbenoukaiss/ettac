mod config;
mod context;
mod error;
pub mod library;
mod runners;
mod ssh;

pub use error::Error;
use std::rc::Rc;

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

    for host in &config.host {
        if !hosts.contains_key(host) {
            panic!("Host {} not found", host);
        }
    }

    for host in &config.host {
        let host = hosts.remove(host).unwrap();

        if let Some(ssh) = &host.ssh {
            let ssh = ssh::login(ssh)?;
        }
    }

    let context = Context {
        host: hosts.remove("prod").unwrap(),
    };

    runner.run(Rc::new(context))?;

    Ok(())
}
