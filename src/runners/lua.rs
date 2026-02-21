use crate::config::Config;
use crate::error::Error;
use crate::runners::{Hosts, Runner, Unknown};
use mlua::prelude::{LuaError, LuaTable};
use mlua::{IntoLua, Lua, Value};
use std::collections::HashMap;
use std::path::Path;
use std::{env, fs};

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

        let env_fn = lua.create_function_mut(|lua, (name, default): (String, Value)| {
            if let Ok(value) = env::var(&name) {
                value.into_lua(&lua)
            } else {
                Ok(default)
            }
        })?;

        globals.set("env", env_fn)?;

        let script = get_script(self.config);
        lua.load(script).set_name(&self.config.script).exec()?;

        Ok(())
    }

    fn get_hosts(&mut self) -> Result<Hosts, Error> {
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

        let defaults = globals.get::<Option<LuaTable>>("defaults")?;

        let mut hosts = HashMap::new();
        for pair in globals
            .get::<LuaTable>("hosts")?
            .pairs::<String, LuaTable>()
        {
            let (name, value) = pair?;

            hosts.insert(name, parse_map(&value, defaults.as_ref())?);
        }

        Ok(hosts)
    }
}

fn get_script(args: &Config) -> String {
    let script_path = Path::new(&args.script);
    if !script_path.exists() {
        panic!("Script {} does not exist", args.script);
    }

    fs::read_to_string(script_path).unwrap()
}

pub fn parse_map(
    table: &LuaTable,
    defaults: Option<&LuaTable>,
) -> Result<HashMap<String, Unknown>, LuaError> {
    parse_map_into(table, defaults, HashMap::new())
}

pub fn parse_vec(table: &LuaTable) -> Result<Vec<Unknown>, LuaError> {
    let mut vec = Vec::new();
    for i in 0..table.len()? {
        vec.push(parse_value(table.get::<Value>(i)?, None)?);
    }

    Ok(vec)
}

fn parse_map_into(
    table: &LuaTable,
    defaults: Option<&LuaTable>,
    mut out: HashMap<String, Unknown>,
) -> Result<HashMap<String, Unknown>, LuaError> {
    let sources = [defaults, Some(table)];
    for table in sources.iter().filter_map(|t| *t) {
        for pair in table.pairs::<String, Value>() {
            let (key, value) = pair?;

            out.insert(key, parse_value(value, defaults)?);
        }
    }

    Ok(out)
}

fn is_array(table: &LuaTable) -> bool {
    let mut i = 0;
    for pair in table.pairs::<Value, Value>() {
        if pair.is_err() || table.get::<Value>(i).is_err() {
            return false;
        }

        i += 1;
    }

    true
}

fn parse_value(value: Value, defaults: Option<&LuaTable>) -> Result<Unknown, LuaError> {
    let parsed = if value.is_null() || value.is_nil() {
        Unknown::None
    } else if value.is_integer() {
        Unknown::Int(value.as_i32().unwrap())
    } else if value.is_boolean() {
        Unknown::Boolean(value.as_boolean().unwrap())
    } else if value.is_number() {
        Unknown::Float(value.as_f32().unwrap())
    } else if value.is_string() {
        Unknown::String(value.as_string().unwrap().to_string_lossy())
    } else if value.is_table() {
        let table = value.as_table().unwrap();
        if is_array(table) {
            Unknown::Vec(parse_vec(table)?)
        } else {
            Unknown::Map(parse_map(table, defaults)?)
        }
    } else {
        panic!("Unparsable value {:?}", value);
    };

    Ok(parsed)
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
