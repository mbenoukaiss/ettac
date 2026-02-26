use std::env;
use crate::context::Context;

pub fn env(name: &str, default: Option<String>) -> Option<String> {
    env::var(name).ok().or(default)
}

pub fn set_timeout(_: &Context, timeout: i32) {
    println!("Setting timeout to {}", timeout);
}

pub fn local(_: &Context, command: &str) {
    println!("Running command locally: {}", command);
}

pub fn remote(_: &Context, command: &str) {
    println!("Running command on remote host: {}", command);
}

pub fn send(_: &Context, from: &str, dest: Option<&str>) {
    println!("Sending file {} to remote host {}", from, dest.unwrap_or(from));
}
