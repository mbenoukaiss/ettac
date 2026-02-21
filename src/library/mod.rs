use std::env;
use crate::runners::Unknown;

pub fn get_env(name: &str, default: Unknown) -> Unknown {
    if let Ok(value) = env::var(name) {
        Unknown::String(value)
    } else {
        default
    }
}
