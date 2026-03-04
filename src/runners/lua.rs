use crate::config::Config;
use crate::context::{AuthMethod, Callable, Context, Host, PartialHost, PartialSshCredentials};
use crate::error::Error;
use crate::library;
use crate::runners::Runner;
use mlua::prelude::{LuaError, LuaTable};
use mlua::{FromLua, Function, IntoLua, Lua, Value};
use partially::Partial;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::rc::Rc;

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

        let script = get_script(self.config)?;
        lua.load(script).set_name(&self.config.script).exec()?;

        let env_fn =
            self.lua
                .create_function(|lua, (name, default): (String, Option<String>)| {
                    library::env(&name, default).into_lua(lua)
                })?;

        globals.set("env", env_fn)?;

        let base64_encode = self.lua.create_function(|lua, (value,): (String,)| {
            library::base64_encode(&value).into_lua(lua)
        })?;

        globals.set("base64_encode", base64_encode)?;

        let base64_decode = self.lua.create_function(|lua, (value,): (String,)| {
            library::base64_decode(&value)
                .map_err(LuaError::from)?
                .into_lua(lua)
        })?;

        globals.set("base64_decode", base64_decode)?;

        Ok(())
    }

    fn get_hosts(&mut self) -> Result<HashMap<String, Host>, Error> {
        let lua = &self.lua;
        let globals = lua.globals();
        globals.set("defaults", lua.create_table()?)?;
        globals.set("hosts", lua.create_table()?)?;

        add_setup_functions(lua)?;
        lua.load("setup()").exec()?;
        remove_setup_functions(lua)?;

        let globals = lua.globals();
        let defaults = if let Ok(table) = globals.get::<LuaTable>("defaults") {
            table
        } else {
            lua.create_table()?
        };

        let defaults = PartialHost::try_from(defaults)?;

        let hosts = globals.get::<LuaTable>("hosts")?;

        let mut parsed_hosts = HashMap::new();
        for pair in hosts.pairs::<String, LuaTable>() {
            let (name, value) = pair?;

            let partial_host = PartialHost::try_from(value)?;
            let mut host_with_defaults = defaults.clone();
            host_with_defaults.apply_some(partial_host);

            let host = Host::try_from(host_with_defaults)?;

            parsed_hosts.insert(name, host);
        }

        Ok(parsed_hosts)
    }

    fn run(&mut self, _: Rc<Context>) -> Result<(), Error> {
        //TODO: populate functions

        Ok(())
    }
}

fn get_script(args: &Config) -> Result<String, Error> {
    let script_path = Path::new(&args.script);
    if !script_path.exists() {
        panic!("Script {} does not exist", args.script);
    }

    Ok(fs::read_to_string(script_path)?)
}

fn add_setup_functions(lua: &Lua) -> Result<(), LuaError> {
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

    Ok(())
}

fn remove_setup_functions(lua: &Lua) -> Result<(), LuaError> {
    let globals = lua.globals();
    globals.raw_remove("default")?;
    globals.raw_remove("host")?;

    Ok(())
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

impl FromLua for LuaFunction {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        if let Value::Function(func) = value {
            Ok(LuaFunction(func))
        } else {
            Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: String::from("function"),
                message: Some(format!("Expected function, got {}", value.type_name())),
            })
        }
    }
}

impl TryFrom<LuaTable> for PartialHost {
    type Error = Error;

    fn try_from(value: LuaTable) -> Result<Self, Self::Error> {
        let ssh = PartialSshCredentials {
            hostname: value.get::<Option<String>>("hostname")?,
            port: value.get::<Option<u16>>("port")?,
            user: value.get::<Option<String>>("user")?,
            credential: if let Some(password) = value.get::<Option<String>>("password")? {
                Some(AuthMethod::Password(password))
            } else if let Some(private_key) = value.get::<Option<String>>("private_key")? {
                Some(AuthMethod::Key(
                    private_key,
                    value.get::<Option<String>>("passphrase")?,
                ))
            } else {
                None
            },
        };

        Ok(PartialHost {
            recipe: value
                .get::<Option<LuaFunction>>("recipe")?
                .map(|lua_fn| Rc::new(lua_fn) as Rc<dyn Callable>),
            repository: value.get::<Option<String>>("repository")?,
            keep_releases: value.get::<Option<i8>>("keep_releases")?,
            persistent_files: value.get::<Option<Vec<String>>>("persistent_files")?,
            persistent_dirs: value.get::<Option<Vec<String>>>("persistent_dirs")?,
            labels: value.get::<Option<Vec<String>>>("labels")?,
            ssh: if !ssh.is_empty() { Some(ssh) } else { None },
            path: value.get::<Option<String>>("path")?,
        })
    }
}
