use std::collections::VecDeque;

use crate::config::Config;

use super::tokenizer::Token;
use error::{Error, ParserError, ParserErrorStack};
use serde::{Deserialize, Serialize};
use textwrap::termwidth;

#[macro_use]
pub mod error;

pub trait Parse {
    fn parse(config: &Config, tokens: &mut VecDeque<Token>) -> Result<Self, ParserError>
    where
        Self: Sized;
    fn check(tokens: &VecDeque<Token>) -> bool
    where
        Self: Sized;
    fn print(&self, config: &Config) -> String;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct File(Vec<Heading>);

impl Parse for File {
    fn parse(config: &Config, tokens: &mut VecDeque<Token>) -> Result<Self, ParserError>
    where
        Self: Sized,
    {
        let mut headings = vec![];

        while !tokens.is_empty() {
            headings.push(error!(Heading::parse(config, tokens), "File")?);
        }

        return Ok(Self(headings));
    }

    fn check(tokens: &VecDeque<Token>) -> bool
    where
        Self: Sized,
    {
        matches!(tokens[1], Token::Colon)
    }

    fn print(&self, config: &Config) -> String {
        format!(
            "{}",
            self.0
                .iter()
                .map(|heading| heading.print(&config))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Heading {
    name: String,
    body: Vec<UnderHeading>,
}

impl Parse for Heading {
    fn parse(config: &Config, tokens: &mut VecDeque<Token>) -> Result<Self, ParserError>
    where
        Self: Sized,
    {
        let name = error!("Heading", tokens);
        let _ = error!("Heading", tokens.pop_front(), [Token::Colon])?;
        let _ = error!("Heading", tokens.pop_front(), [Token::Newline])?;
        let mut body = vec![];

        loop {
            if tokens.is_empty() {
                break;
            }
            if tokens[0] == Token::Newline {
                tokens.pop_front();
                break;
            }

            if Todo::check(&tokens) {
                body.push(UnderHeading::Todo(error!(
                    Todo::parse(config, tokens),
                    "Heading"
                )?))
            } else if Bullet::check(&tokens) {
                body.push(UnderHeading::Bullet(error!(Bullet::parse(config, tokens), "Heading")?));
                let _ = error!("Heading", tokens.pop_front(), [Token::Newline])?;
            } else if Text::check(&tokens) {
                body.push(UnderHeading::Text(error!(
                    Text::parse(config, tokens),
                    "Heading"
                )?));
                let _ = error!("Heading", tokens.pop_front(), [Token::Newline])?;
            } else if Heading::check(&tokens) {
                return Err(error!(
                    "Heading",
                    Error::Other(format!("Can't have a heading in a heading"))
                ));
            }
        }

        Ok(Self { name, body })
    }

    fn check(tokens: &VecDeque<Token>) -> bool
    where
        Self: Sized,
    {
        matches!(tokens[1], Token::Colon)
    }

    fn print(&self, config: &Config) -> String {
        let mut buf = format!("{}\n", self.name);

        for body in &self.body {
            if let UnderHeading::Text(text) = body {
                buf = format!("{buf}{}", text.print(&config));
                continue;
            }
            buf = match body {
                    UnderHeading::Todo(todo) => format!("{buf}    {}\n", todo.print(&config)),
                    UnderHeading::Bullet(bullet) => format!("{buf}    {}\n", bullet.print(&config)),
                    UnderHeading::Text(text) => format!("{buf}{}\n", text.print(&config)),
                };
        }

        return buf;
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum UnderHeading {
    Todo(Todo),
    Bullet(Bullet),
    Text(Text),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Todo {
    state: Option<TodoState>,
    description: String,
}

impl Parse for Todo {
    fn parse(config: &Config, tokens: &mut VecDeque<Token>) -> Result<Self, ParserError>
    where
        Self: Sized,
    {
        let _ = error!("Todo", tokens.pop_front(), [Token::BracketOpen])?;
        let state = if TodoState::check(&tokens) {
            Some(error!(TodoState::parse(config, tokens), "Todo")?)
        } else {
            None
        };
        let _ = error!("Todo", tokens.pop_front(), [Token::BracketClose])?;
        let description = error!("Todo", tokens);
        let _ = error!("Todo", tokens.pop_front(), [Token::Newline])?;

        Ok(Self { state, description })
    }

    fn check(tokens: &VecDeque<Token>) -> bool
    where
        Self: Sized,
    {
        matches!(tokens[0], Token::BracketOpen)
    }

    fn print(&self, config: &Config) -> String {
        let (brackets, state) = if let Some(state) = &self.state {
            (false, state.print(&config))
        } else if let Some(ops) = &config.todo_state_ops {
            (ops.brackets, ops.default.to_owned())
        } else {
            (false, " ".to_owned())
        };

        if brackets {
            format!("[{state}] {}", self.description)
        } else {
            format!("{state} {}", self.description)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Bullet(String);

impl Parse for Bullet {
    fn parse(_: &Config, tokens: &mut VecDeque<Token>) -> Result<Self, ParserError>
        where
            Self: Sized {
        let _ = error!("Bullet", tokens.pop_front(), [Token::Dash])?;
        Ok(Self(error!("Bullet", tokens)))
    }

    fn check(tokens: &VecDeque<Token>) -> bool
        where
            Self: Sized {
        matches!(tokens[0], Token::Dash)
    }

    fn print(&self, config: &Config) -> String {
        if let Some(bullet) = &config.bullet_point {
            format!("{bullet} {}", self.0)
        } else {
            format!("- {}", self.0)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Text(String);

impl Parse for Text {
    fn parse(_: &Config, tokens: &mut VecDeque<Token>) -> Result<Self, ParserError>
    where
        Self: Sized,
    {
        Ok(Self(error!("Text", tokens)))
    }

    fn check(tokens: &VecDeque<Token>) -> bool
    where
        Self: Sized,
    {
        matches!(tokens[0], Token::Text(_))
    }

    fn print(&self, _: &Config) -> String {
        textwrap::indent(&textwrap::fill(&self.0, termwidth() - 4), "    ") + "\n"
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum TodoState {
    Defined(String),
    Other(String),
}

impl Parse for TodoState {
    fn parse(config: &Config, tokens: &mut VecDeque<Token>) -> Result<Self, ParserError>
    where
        Self: Sized,
    {
        let str = error!("TodoState", tokens);

        Ok(if let Some(state) = config.todo_state.get(&str) {
            Self::Defined(state.to_owned())
        } else {
            Self::Other(str.to_owned())
        })
    }

    fn check(tokens: &VecDeque<Token>) -> bool
    where
        Self: Sized,
    {
        matches!(
            (&tokens[0], &tokens[1]),
            (Token::Text(_), Token::BracketClose)
        )
    }

    fn print(&self, _: &Config) -> String {
        match self {
            Self::Defined(str) => str,
            Self::Other(str) => str,
        }
        .to_owned()
    }
}
