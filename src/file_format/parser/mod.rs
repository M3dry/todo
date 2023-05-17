use std::{collections::VecDeque};

use crate::config::Config;

use super::tokenizer::{Token, TextToken};
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
        matches!(tokens[0], Token::Heading(_))
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
        let name = error!("Heading", tokens, Heading);
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
                    PrintText::parse(config, tokens),
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
        matches!(tokens[0], Token::Heading(_))
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
    Text(PrintText),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Todo {
    state: TodoState,
    description: Text,
}

impl Parse for Todo {
    fn parse(config: &Config, tokens: &mut VecDeque<Token>) -> Result<Self, ParserError>
    where
        Self: Sized,
    {
        let _ = error!("Todo", tokens.pop_front(), [Token::BracketOpen])?;
        let state = error!(TodoState::parse(config, tokens), "Todo")?;
        let _ = error!("Todo", tokens.pop_front(), [Token::BracketClose])?;
        let description = error!(Text::parse(config, tokens), "Todo")?;
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
        let brackets = if let Some(ops) = &config.todo_state_ops {
            ops.brackets
        } else {
            true
        };
        let state = if self.state.empty() {
            " ".to_owned()
        } else {
            self.state.print(config)
        };

        if brackets {
            format!("[{state}] {}", self.description.print(config))
        } else {
            format!("{state} {}", self.description.print(config))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum TodoState {
    Defined(String),
    Other(String),
}

impl TodoState {
    fn empty(&self) -> bool {
        match self {
            Self::Defined(str) | Self::Other(str) => str.is_empty(),
        }
    }
}

impl Parse for TodoState {
    fn parse(config: &Config, tokens: &mut VecDeque<Token>) -> Result<Self, ParserError>
    where
        Self: Sized,
    {
        let str = if let Token::Inside(text) = error!("TodoState", tokens.pop_front(), [Token::Inside(_)])? {
            text
        } else {
            unreachable!()
        };

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
            (Token::Inside(_), Token::BracketClose)
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

#[derive(Debug, Serialize, Deserialize)]
struct Bullet(Text);

impl Parse for Bullet {
    fn parse(config: &Config, tokens: &mut VecDeque<Token>) -> Result<Self, ParserError>
        where
            Self: Sized {
        Ok(Self(error!(Text::parse(config, tokens), "Bullet")?))
    }

    fn check(tokens: &VecDeque<Token>) -> bool
        where
            Self: Sized {
        matches!(tokens[0], Token::Bullet(_))
    }

    fn print(&self, config: &Config) -> String {
        if let Some(bullet) = &config.bullet_point {
            format!("{bullet} {}", self.0.print(config))
        } else {
            format!("- {}", self.0.print(config))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct PrintText(Text);

impl Parse for PrintText {
    fn parse(config: &Config, tokens: &mut VecDeque<Token>) -> Result<Self, ParserError>
        where
            Self: Sized {
        Ok(Self(Text::parse(config, tokens)?))
    }
    fn check(tokens: &VecDeque<Token>) -> bool
        where
            Self: Sized {
        Text::check(tokens)
    }

    fn print(&self, config: &Config) -> String {
        textwrap::indent(&textwrap::fill(&self.0.print(config), termwidth() - 4), "    ") + "\n"
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Text(Vec<TextOps>);

impl Parse for Text {
    fn parse(_: &Config, tokens: &mut VecDeque<Token>) -> Result<Self, ParserError>
        where
            Self: Sized {
        Ok(Self(match error!("Text", tokens.pop_front(), [Token::Bullet(_), Token::Text(_)])? {
            Token::Bullet(ops) | Token::Text(ops) => ops.to_vecdeque(),
            _ => unreachable!(),
        }.into_iter().map(|op|TextOps::from(op)).collect()))
    }

    fn check(tokens: &VecDeque<Token>) -> bool
        where
            Self: Sized {
        matches!(tokens[0], Token::Text(_) | Token::Bullet(_))
    }

    fn print(&self, _: &Config) -> String {
        self.0.iter().map(|op| op.to_string()).collect::<Vec<String>>().join("")
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum TextOps {
    Verbatim(String),
    Underline(String),
    Crossed(String),
    Bold(String),
    Italic(String),
    Normal(String),
}

impl From<TextToken> for TextOps {
    fn from(value: TextToken) -> Self {
        match value {
            TextToken::Verbatim(str) => Self::Verbatim(str),
            TextToken::Underline(str) => Self::Underline(str),
            TextToken::Crossed(str) => Self::Crossed(str),
            TextToken::Bold(str) => Self::Bold(str),
            TextToken::Italic(str) => Self::Italic(str),
            TextToken::Text(str) => Self::Normal(str),
        }
    }
}

impl std::fmt::Display for TextOps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Verbatim(str) => "`".to_owned() + str + "`",
            Self::Underline(str) => "_".to_owned() + str + "_",
            Self::Crossed(str) => "-".to_owned() + str + "-",
            Self::Bold(str) => "*".to_owned() + str + "*",
            Self::Italic(str) => "/".to_owned() + str + "/",
            Self::Normal(str) => str.to_owned(),
        })
    }
}
