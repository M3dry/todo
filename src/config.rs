use std::{collections::{HashMap, HashSet}, path::PathBuf};

use mlua::{Function, Lua, Result as LuaResult, Table};
use serde::{Deserialize, Serialize};
use shellexpand::tilde;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub template: Option<PathBuf>,
    pub directory: PathBuf,
    pub editor: Option<String>,
    pub bullet_point: Option<String>,
    pub todo_state_ops: Option<TodoStateOps>,
    pub todo_state: HashMap<String, String>,
    pub link_handlers: HashSet<String>,
}

impl Config {
    pub fn get() -> LuaResult<Self> {
        let config = xdg::BaseDirectories::with_prefix("todo").unwrap();
        let config_path = config.place_config_file("config.lua").unwrap();

        if !config_path.exists() {
            std::fs::write(
                &config_path,
                r#"return {
    directory = "~/todo",
}"#,
            )
            .unwrap();
        }

        Ok({
            let lua = Lua::new();
            let mut config = Self::from_table(
                lua.load(&std::fs::read_to_string(&config_path).unwrap())
                    .eval::<Table>()?,
            )?;
            if let Some(template) = &mut config.template {
                *template = PathBuf::from(tilde(template.to_str().unwrap()).to_string());
            }
            config.directory = PathBuf::from(tilde(config.directory.to_str().unwrap()).to_string());

            config
        })
    }

    pub fn get_handlers<'lua>(lua: &'lua Lua) -> LuaResult<HashMap<String, Function<'lua>>> {
        let config = xdg::BaseDirectories::with_prefix("todo").unwrap();
        let config_path = config.place_config_file("config.lua").unwrap();

        if !config_path.exists() {
            std::fs::write(
                &config_path,
                r#"return {
    directory = "~/todo",
}"#,
            )
            .unwrap();
        }

        Ok({
            let mut table = lua.load(&std::fs::read_to_string(&config_path).unwrap()).eval::<Table>()?;

            if let Some(table) = table.get::<_, Option<Table>>("link_handlers")? {
                HashMap::from_iter(table.pairs::<String, Function>().into_iter().filter_map(|pair| pair.ok()))
            } else {
                HashMap::new()
            }
        })
    }

    fn from_table(table: Table) -> LuaResult<Self> {
        Ok(Self {
            template: table
                .get::<_, String>("template")
                .ok()
                .map(|template| PathBuf::from(template)),
            directory: PathBuf::from(table.get::<_, String>("directory")?),
            editor: table.get("editor").ok(),
            bullet_point: table.get("bullet_point").ok(),
            todo_state_ops: if let Some(table) = table.get::<_, Table>("todo_state_ops").ok() {
                Some(TodoStateOps::from_table(table)?)
            } else {
                None
            },
            todo_state: if let Some(table) = table.get::<_, Option<Table>>("todo_state")? {
                HashMap::from_iter(
                    table
                        .pairs::<String, String>()
                        .into_iter()
                        .filter_map(|pair| pair.ok()),
                )
            } else {
                HashMap::new()
            },
            link_handlers: if let Some(table) = table.get::<_, Option<Table>>("link_handlers")? {
                HashSet::from_iter(
                    table
                        .pairs::<String, Function>()
                        .into_iter()
                        .filter_map(|pair| match pair {
                            Ok((key, _)) => Some(key),
                            _ => None,
                        }),
                )
            } else {
                HashSet::new()
            },
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TodoStateOps {
    pub default: String,
    pub brackets: bool,
}

impl TodoStateOps {
    fn from_table(table: Table) -> LuaResult<Self> {
        Ok(Self {
            default: table.get::<_, String>("default")?,
            brackets: table.get::<_, bool>("brackets").unwrap_or(true),
        })
    }
}
