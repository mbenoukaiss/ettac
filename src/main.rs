mod config;
mod context;
mod error;
mod library;
mod runners;
mod ssh;

use error::Error;
use std::rc::Rc;

use crate::config::Config;
use crate::context::{Context, Host};
use crate::runners::{LuaRunner, Runner};

fn main() -> Result<(), Error> {
    let config = argh::from_env::<Config>();
    let config = Box::leak(Box::new(config)) as &'static Config;

    let mut runner = LuaRunner::new(config);
    runner.init()?;

    let hosts = runner.get_hosts()?;
    let unknown_hosts = config.hosts.iter()
        .cloned()
        .filter(|host| !hosts.contains_key(host))
        .collect::<Vec<String>>();
println!("{:?}", unknown_hosts);
    if !unknown_hosts.is_empty() {
        return Err(Error::UnknownHosts(unknown_hosts));
    }

    let hosts = hosts
        .into_iter()
        .filter(|(name, _)| config.hosts.contains(name))
        .collect::<Vec<(String, Host)>>();

    for (name, host) in hosts {
        println!("Deploying host {}", name);

        if let Some(ssh) = &host.ssh {
            let _ = ssh::login(ssh)?;
        }

        let context = Context { host };

        runner.run(Rc::new(context))?;
    }

    Ok(())
}
