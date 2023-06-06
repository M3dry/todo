use std::{fmt::Display, path::PathBuf, process::Stdio};

use chrono::{Duration, Local};
use clap::{Parser, Subcommand, ValueEnum};
use config::Config;
use file_format::{
    parser::{self, Handler, Parse},
    tokenizer::Tokens,
};
use mlua::Lua;

mod config;
mod file_format;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(value_enum)]
    day: Option<Day>,
    #[arg(short, long)]
    file: Option<String>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Day {
    #[value(name = "y")]
    Yesterday,
    #[value(name = "t")]
    Today,
    #[value(name = "tmr")]
    Tomorrow,
}

impl Display for Day {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Yesterday => "yesterday",
                Self::Today => "today",
                Self::Tomorrow => "tomorrow",
            }
        )
    }
}

#[derive(Subcommand)]
enum Command {
    New {
        #[arg(short, long)]
        editor: bool,
    },
    Edit,
    Show,
    Raw,
    EwwShow,
    OpenLink {
        id: usize,
    },
    OpenLinkRaw {
        handler: String,
        path: String,
    },
    ListLinks,
    Config,
    Tokens,
}

fn main() {
    let arg = Args::parse();
    let config = Config::get().unwrap();
    let mut file = (&config.directory).clone();
    let day = if let Some(day) = arg.day {
        file.push(
            match day {
                Day::Yesterday => Local::now() - Duration::days(1),
                Day::Today => Local::now(),
                Day::Tomorrow => Local::now() + Duration::days(1),
            }
            .format("%d%m%Y.todo")
            .to_string(),
        );
        true
    } else if let Some(file_) = arg.file {
        file.push(file_ + ".todo");
        false
    } else {
        return;
    };
    let exists = std::path::Path::new(&file).exists();

    match arg.command {
        Command::New { .. } if day && exists && arg.day.is_some() => {
            eprintln!("Todo for {} already exists", arg.day.unwrap())
        }
        Command::New { editor: true } => {
            let template = if !day {
                "".to_string()
            } else if let Some(template) = get_template(&arg.day, &config) {
                let template = std::fs::read_to_string(&template).unwrap();
                template
            } else {
                "".to_string()
            };
            std::fs::write(&file, template).unwrap();

            if let Some(editor) = &config.editor {
                std::process::Command::new(&editor)
                    .arg(&file)
                    .stdin(Stdio::inherit())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .output()
                    .unwrap();
            } else {
                edit::edit_file(&file).unwrap();
            }
        }
        Command::New { .. } => {
            std::fs::write(
                &file,
                if !day {
                    "".to_string()
                } else if let Some(template) = get_template(&arg.day, &config) {
                    let template = std::fs::read_to_string(&template).unwrap();
                    template
                } else {
                    "".to_string()
                },
            )
            .unwrap();
        }
        Command::Edit if exists => {
            if let Some(editor) = &config.editor {
                std::process::Command::new(&editor)
                    .arg(&file)
                    .stdin(Stdio::inherit())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .output()
                    .unwrap();
            } else {
                edit::edit_file(&file).unwrap();
            }
        }
        Command::Show if exists => {
            let tokens: Tokens = std::fs::read_to_string(&file).unwrap().parse().unwrap();
            let mut vecdeque = tokens.to_vecdeque();

            print!(
                "{}",
                match parser::File::parse(&config, &mut vecdeque) {
                    Ok(ok) => ok.print(&config),
                    Err(err) => err.to_string(),
                }
            );
        }
        Command::Raw if exists => {
            let tokens: Tokens = std::fs::read_to_string(&file).unwrap().parse().unwrap();
            let mut vecdeque = tokens.to_vecdeque();

            print!(
                "{}",
                match parser::File::parse(&config, &mut vecdeque) {
                    Ok(ok) => serde_json::to_string_pretty(&ok).unwrap(),
                    Err(err) => err.to_string(),
                }
            );
        }
        Command::EwwShow if exists => {
            let tokens: Tokens = std::fs::read_to_string(&file).unwrap().parse().unwrap();
            let mut vecdeque = tokens.to_vecdeque();

            println!(
                "{}",
                match parser::File::parse(&config, &mut vecdeque) {
                    Ok(ok) => serde_json::to_string_pretty(&file_format::eww::EwwTodo::from_todos(
                        ok.headings()
                            .into_iter()
                            .flat_map(|heading| heading.todos())
                            .collect(),
                        &config
                    ))
                    .unwrap(),
                    Err(err) => err.to_string(),
                }
            )
        }
        Command::ListLinks if exists => {
            let mut tokens = std::fs::read_to_string(&file)
                .unwrap()
                .parse::<Tokens>()
                .unwrap()
                .to_vecdeque();

            println!(
                "{}",
                match parser::File::parse(&config, &mut tokens) {
                    Ok(ok) => {
                        ok.headings()
                            .into_iter()
                            .flat_map(|heading| {
                                heading.links().into_iter().map(|link| {
                                    let (name, handler, path) = link;

                                    format!("{name} - {handler}:{path}")
                                })
                            })
                            .enumerate()
                            .map(|(i, str)| format!("{i} {str}"))
                            .collect()
                    }
                    Err(err) => err.to_string(),
                }
            )
        }
        Command::OpenLink { id } => {
            let mut tokens = std::fs::read_to_string(&file)
                .unwrap()
                .parse::<Tokens>()
                .unwrap()
                .to_vecdeque();

            match parser::File::parse(&config, &mut tokens) {
                Ok(ok) => {
                    let links = ok
                        .headings()
                        .into_iter()
                        .flat_map(|heading| heading.links())
                        .collect::<Vec<(&String, &Handler, &String)>>();

                    if let Some(link) = links.get(id) {
                        let lua = Lua::new();

                        link.1
                            .open(link.2.to_string(), Config::get_handlers(&lua).unwrap())
                    } else {
                        eprintln!("Id is out of bounds, max is {}", links.len() - 1)
                    }
                }
                Err(err) => println!("{}", err.to_string()),
            }
        }
        Command::OpenLinkRaw { handler, path } => {
            let lua = Lua::new();
            let handler = file_format::parser::Handler::from((
                file_format::tokenizer::Handler(handler),
                &config,
            ));

            handler.open(path, Config::get_handlers(&lua).unwrap())
        }
        Command::Config => {
            let config = Config::get().unwrap();
            println!("{}", serde_json::to_string_pretty(&config).unwrap());
        }
        Command::Tokens if exists => {
            let tokens: Tokens = std::fs::read_to_string(&file).unwrap().parse().unwrap();
            let vecdeque = tokens.to_vecdeque();

            println!("{vecdeque:#?}")
        }
        _ => eprintln!("File doesn't exist"),
    }
}

fn get_template<'a>(day: &Option<Day>, config: &'a Config) -> &'a Option<PathBuf> {
    if matches!(day, Some(Day::Tomorrow)) {
        &config.template_tmr
    } else {
        &config.template
    }
}
