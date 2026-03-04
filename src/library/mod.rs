use std::env;
use base64::prelude::*;
use crate::context::Context;
use crate::Error;

pub fn env(name: &str, default: Option<String>) -> Option<String> {
    env::var(name).ok().or(default)
}

pub fn base64_encode(value: &str) -> String {
    BASE64_STANDARD.encode(value.as_bytes())
}

pub fn base64_decode(value: &str) -> Result<String, Error> {
    let decoded = BASE64_STANDARD.decode(value).map_err(|_| Error::InvalidBase64(value.to_string()))?;
    let string = String::from_utf8(decoded).map_err(|_| Error::InvalidBase64(value.to_string()))?;

    Ok(string)
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
