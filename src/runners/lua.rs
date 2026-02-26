use crate::config::Config;
use crate::context::{Callable, Context, Host, Ssh, SshCredential};
use crate::error::Error;
use crate::runners::Runner;
use mlua::prelude::{LuaError, LuaTable};
use mlua::{Function, IntoLua, Lua, Value};
use std::collections::HashMap;
use std::path::Path;
use std::fs;
use std::rc::Rc;
use crate::library;

pub struct LuaRunner {
    lua: Lua,
    config: &'static Config,
}

impl LuaRunner {
    pub fn new(config: &'static Config) -> Self {
        Self {
            lua: Lua::new(),
            config,
        }
    }
}

impl Runner for LuaRunner {
    fn init(&mut self) -> Result<(), Error> {
        let lua = &mut self.lua;
        let globals = lua.globals();

        let script = get_script(self.config);
        lua.load(script).set_name(&self.config.script).exec()?;

        let env_fn = self.lua.create_function(|lua, (name, default): (String, Option<String>)| {
            library::env(&name, default).into_lua(lua)
        })?;

        globals.set("env", env_fn)?;

        Ok(())
    }

    fn get_hosts(&mut self) -> Result<HashMap<String, Host>, Error> {
        let lua = &mut self.lua;
        let globals = lua.globals();
        globals.set("defaults", lua.create_table()?)?;
        globals.set("hosts", lua.create_table()?)?;

        let default_fn = lua.create_function_mut(|lua, (data,): (LuaTable,)| {
            let globals = lua.globals();
            let current_defaults = if globals.contains_key("defaults")? {
                globals.get::<LuaTable>("defaults")?
            } else {
                lua.create_table()?
            };

            globals.set("defaults", merge_tables(&lua, &current_defaults, &data)?)?;

            Ok(())
        })?;

        globals.set("default", default_fn)?;

        let host_fn = lua.create_function_mut(|lua, (name, data): (String, LuaTable)| {
            let hosts = lua.globals().get::<LuaTable>("hosts")?;
            hosts.set(name, data)?;

            Ok(())
        })?;

        globals.set("host", host_fn)?;

        lua.load("setup()").exec()?;

        let defaults = if let Ok(table) = globals.get::<LuaTable>("defaults") {
            table
        } else {
            lua.create_table()?
        };

        let default_recipe = defaults.get::<Option<Function>>("recipe")?;
        let default_repository = defaults.get::<Option<String>>("repository")?;
        let default_keep_releases = defaults.get::<Option<i8>>("keep_releases")?;
        let default_persistent_files = defaults.get::<Option<Vec<String>>>("persistent_files")?;
        let default_persistent_dirs = defaults.get::<Option<Vec<String>>>("persistent_dirs")?;
        let default_labels = defaults.get::<Option<Vec<String>>>("labels")?;

        let default_hostname = defaults.get::<Option<String>>("hostname")?;
        let default_port = defaults.get::<Option<u16>>("port")?;
        let default_user = defaults.get::<Option<String>>("user")?;
        let default_private_key = defaults.get::<Option<String>>("private_key")?;
        let default_password = defaults.get::<Option<String>>("password")?;
        let default_path = defaults.get::<Option<String>>("path")?;

        let hosts = globals.get::<LuaTable>("hosts")?;

        let mut parsed_hosts = HashMap::new();
        for pair in hosts.pairs::<String, LuaTable>() {
            let (name, value) = pair?;

            let recipe = value.get::<Option<Function>>("recipe")?;
            let repository = value.get::<Option<String>>("repository")?;
            let keep_releases = value.get::<Option<i8>>("keep_releases")?;
            let persistent_files = value.get::<Option<Vec<String>>>("persistent_files")?;
            let persistent_dirs = value.get::<Option<Vec<String>>>("persistent_dirs")?;
            let labels = value.get::<Option<Vec<String>>>("labels")?;

            let hostname = value.get::<Option<String>>("hostname")?;
            let port = value.get::<Option<u16>>("port")?;
            let user = value.get::<Option<String>>("user")?;
            let private_key = value.get::<Option<String>>("private_key")?;
            let password = value.get::<Option<String>>("password")?;
            let path = value.get::<Option<String>>("path")?;

            let hostname = hostname.or(default_hostname.clone());
            let port = port.or(default_port.clone()).unwrap_or(22);
            let user = user.or(default_user.clone());
            let private_key = private_key.or(default_private_key.clone());
            let password = password.or(default_password.clone());

            let has_ssh_arguments =
                hostname.is_some() || user.is_some() || private_key.is_some() || password.is_some();
            let not_all_ssh_arguments = hostname.is_none()
                || user.is_none()
                || (private_key.is_none() && password.is_none());
            if has_ssh_arguments && not_all_ssh_arguments {
                return Err(Error::config(
                    "hostname, user and private_key or password must be set together",
                ));
            }

            let ssh = if has_ssh_arguments {
                Some(Ssh {
                    hostname: hostname.unwrap(),
                    port,
                    user: user.unwrap(),
                    credential: if private_key.is_some() {
                        SshCredential::PrivateKey(private_key.unwrap())
                    } else {
                        SshCredential::Password(password.unwrap())
                    },
                })
            } else {
                None
            };

            let host = Host {
                recipe: Box::new(LuaFunction(
                    recipe
                        .or(default_recipe.clone())
                        .ok_or_else(|| Error::config("recipe is required"))?,
                )),
                repository: repository
                    .or(default_repository.clone())
                    .ok_or_else(|| Error::config("repository is required"))?,
                keep_releases: keep_releases.or(default_keep_releases).unwrap_or(3),
                persistent_files: persistent_files
                    .or(default_persistent_files.clone())
                    .unwrap_or_default(),
                persistent_dirs: persistent_dirs
                    .or(default_persistent_dirs.clone())
                    .unwrap_or_default(),
                labels: labels.or(default_labels.clone()).unwrap_or_default(),
                ssh,
                path: path
                    .or(default_path.clone())
                    .ok_or_else(|| Error::config("path is required"))?,
            };

            parsed_hosts.insert(name, host);
        }

        Ok(parsed_hosts)
    }

    fn run(&mut self, ctx: Rc<Context>) -> Result<(), Error> {
        let lua = &mut self.lua;
        let globals = lua.globals();

        //TODO: populate functions

        Ok(())
    }
}

fn get_script(args: &Config) -> String {
    let script_path = Path::new(&args.script);
    if !script_path.exists() {
        panic!("Script {} does not exist", args.script);
    }

    fs::read_to_string(script_path).unwrap()
}

fn merge_tables(lua: &Lua, a: &LuaTable, b: &LuaTable) -> Result<LuaTable, LuaError> {
    let out = a.clone();

    for pair in b.pairs::<String, Value>() {
        let (key, b_value) = pair?;
        let key_ref = key.as_str();

        out.set(key_ref, &b_value)?;

        if !b_value.is_table() {
            continue;
        }

        let b_table = b_value.as_table().unwrap();
        if is_array(b_table) {
            continue;
        }

        let a_value = out.get::<Value>(key_ref)?;
        if !a_value.is_table() {
            continue;
        }

        let a_table = a_value.as_table().unwrap();
        if is_array(a_table) {
            continue;
        }

        //both values are table so we can merge them
        out.set(key_ref, merge_tables(lua, a_table, b_table)?)?;
    }

    Ok(out)
}

fn is_array(table: &LuaTable) -> bool {
    let mut i = 1;
    for pair in table.pairs::<Value, Value>() {
        if pair.is_err() || table.get::<Value>(i).is_err() {
            return false;
        }

        i += 1;
    }

    let Ok(len) = table.len() else {
        return false;
    };

    i == len + 1
}

#[derive(Debug)]
struct LuaFunction(Function);

impl Callable for LuaFunction {
    fn call(&self) -> Result<(), Error> {
        Ok(self.0.call(())?)
    }
}
