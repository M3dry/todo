use std::{fmt::Display, process::Stdio};

use chrono::{Duration, Local};
use clap::{Parser, Subcommand, ValueEnum};
use config::Config;
use file_format::{
    parser::{self, Parse},
    tokenizer::Tokens,
};

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
    Config,
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
            } else if let Some(template) = &config.template {
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
                } else if let Some(template) = &config.template {
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
        Command::Config => {
            let config = Config::get().unwrap();
            println!("{}", serde_json::to_string_pretty(&config).unwrap());
        }
        _ => eprintln!("File doesn't exist"),
    }
}
