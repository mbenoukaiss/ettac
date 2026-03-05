#![feature(try_trait_v2)]

//while not fully implemented
#![allow(dead_code)]

mod access;
mod config;
mod context;
mod error;
mod library;
mod runners;

use error::Error;

use crate::config::Config;
use crate::context::{Context, Host};
use crate::runners::{LuaRunner, Runner};

fn main() {
    let config = argh::from_env::<Config>();
    let config = Box::leak(Box::new(config)) as &'static Config;

    if let Err(err) = with_lua(config) {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

fn with_lua(config: &'static Config) -> Result<(), Error> {
    let mut runner = LuaRunner::new(config);
    runner.init()?;

    let hosts = runner.get_hosts()?;
    let unknown_hosts = config
        .hosts
        .iter()
        .filter(|host| !hosts.contains_key(*host))
        .cloned()
        .collect::<Vec<String>>();

    if !unknown_hosts.is_empty() {
        Error::UnknownHosts(unknown_hosts)?
    }

    let hosts = hosts
        .into_iter()
        .filter(|(name, _)| config.hosts.contains(name))
        .collect::<Vec<(String, Host)>>();

    for (name, host) in hosts {
        println!("Deploying host {}", name);

        let access = access::to(&host.path, &host.ssh)?;
        let context = Context { host, access };

        runner.run(context)?;
    }

    Ok(())
}
