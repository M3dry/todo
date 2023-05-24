use std::collections::{HashMap, VecDeque};

use crate::config::Config;

use super::tokenizer::{TextToken, Token};
use error::{Error, ParserError, ParserErrorStack};
use mlua::Function;
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

impl File {
    pub fn headings(&self) -> &Vec<Heading> {
        &self.0
    }
}

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
pub struct Heading {
    name: String,
    body: Vec<UnderHeading>,
}

impl Heading {
    pub fn todos(&self) -> Vec<&Todo> {
        self.body
            .iter()
            .filter_map(|under| match under {
                UnderHeading::Todo(todo) => Some(todo),
                _ => None,
            })
            .collect()
    }

    pub fn links(&self) -> Vec<(&String, &Handler, &String)> {
        self.body
            .iter()
            .flat_map(|under| {
                let Text(text_ops) = match under {
                    UnderHeading::Text(PrintText(text)) => text,
                    UnderHeading::Bullet(Bullet { text, .. }) => text,
                    UnderHeading::Todo(Todo { description, .. }) => description,
                };
                text_ops
                    .into_iter()
                    .filter_map(|op| {
                        if let TextOp::Link {
                            name,
                            handler,
                            path,
                        } = op
                        {
                            Some((name, handler, path))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<(&String, &Handler, &String)>>()
            })
            .collect()
    }
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
                body.push(UnderHeading::Bullet(error!(
                    Bullet::parse(config, tokens),
                    "Heading"
                )?));
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
    pub state: TodoState,
    pub description: Text,
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
            if let Some(ops) = &config.todo_state_ops {
                ops.default.to_owned()
            } else {
                " ".to_owned()
            }
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
pub enum TodoState {
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
        let str = if let Token::Inside(text) =
            error!("TodoState", tokens.pop_front(), [Token::Inside(_)])?
        {
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

    fn print(&self, config: &Config) -> String {
        let str = match self {
            Self::Defined(str) => str,
            Self::Other(str) => str,
        };
        let brackets = if let Some(ops) = &config.todo_state_ops {
            ops.brackets
        } else {
            true
        };
        let state = if str.is_empty() {
            if let Some(ops) = &config.todo_state_ops {
                &ops.default
            } else {
                " "
            }
        } else {
            str
        };

        if brackets {
            format!("[{state}]")
        } else {
            format!("{state}")
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Bullet {
    bullet: bool,
    text: Text,
}

impl Parse for Bullet {
    fn parse(config: &Config, tokens: &mut VecDeque<Token>) -> Result<Self, ParserError>
    where
        Self: Sized,
    {
        Ok(Self {
            bullet: true,
            text: error!(Text::parse(config, tokens), "Bullet")?,
        })
    }

    fn check(tokens: &VecDeque<Token>) -> bool
    where
        Self: Sized,
    {
        matches!(tokens[0], Token::Bullet(_))
    }

    fn print(&self, config: &Config) -> String {
        if let Some(bullet) = &config.bullet_point {
            format!("{bullet} {}", self.text.print(config))
        } else {
            format!("- {}", self.text.print(config))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct PrintText(Text);

impl PrintText {
    pub fn text(&self) -> &Text {
        &self.0
    }
}

impl Parse for PrintText {
    fn parse(config: &Config, tokens: &mut VecDeque<Token>) -> Result<Self, ParserError>
    where
        Self: Sized,
    {
        Ok(Self(Text::parse(config, tokens)?))
    }
    fn check(tokens: &VecDeque<Token>) -> bool
    where
        Self: Sized,
    {
        Text::check(tokens)
    }

    fn print(&self, config: &Config) -> String {
        textwrap::indent(
            &textwrap::fill(&self.0.print(config), termwidth() - 4),
            "    ",
        ) + "\n"
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Text(pub Vec<TextOp>);

impl Parse for Text {
    fn parse(config: &Config, tokens: &mut VecDeque<Token>) -> Result<Self, ParserError>
    where
        Self: Sized,
    {
        Ok(Self(
            match error!(
                "Text",
                tokens.pop_front(),
                [Token::Bullet(_), Token::Text(_)]
            )? {
                Token::Bullet(ops) | Token::Text(ops) => ops.to_vecdeque(),
                _ => unreachable!(),
            }
            .into_iter()
            .map(|op| TextOp::from((op, config)))
            .collect(),
        ))
    }

    fn check(tokens: &VecDeque<Token>) -> bool
    where
        Self: Sized,
    {
        matches!(tokens[0], Token::Text(_) | Token::Bullet(_))
    }

    fn print(&self, _: &Config) -> String {
        self.0
            .iter()
            .map(|op| op.to_string())
            .collect::<Vec<String>>()
            .join("")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Handler {
    Custom(String),
    Unknown(String),
}

impl Handler {
    pub fn open<'lua>(&self, path: String, handlers: HashMap<String, Function<'lua>>) {
        match self {
            Self::Custom(str) => {
                if let Some(func) = handlers.get(str) {
                    func.call::<_, ()>(path).unwrap();
                }
            }
            Self::Unknown(str) => panic!(
                "cant find link handler for {str:?}, also I need to do better error handling"
            ),
        }
    }

    pub fn to_string(&self) -> &String {
        match self {
            Self::Custom(str) | Self::Unknown(str) => &str,
        }
    }
}

impl From<(super::tokenizer::Handler, &Config)> for Handler {
    fn from((value, config): (super::tokenizer::Handler, &Config)) -> Self {
        use super::tokenizer::Handler as THandler;

        match value {
            THandler(str) if config.link_handlers.contains(&str) => Self::Custom(str),
            THandler(str) => Self::Unknown(str),
        }
    }
}

impl std::fmt::Display for Handler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Custom(str) | Self::Unknown(str) => str.as_str(),
            }
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TextOp {
    Verbatim(Vec<TextOp>),
    Underline(Vec<TextOp>),
    Crossed(Vec<TextOp>),
    Bold(Vec<TextOp>),
    Italic(Vec<TextOp>),
    Link {
        name: String,
        handler: Handler,
        path: String,
    },
    TextExtra(char, Vec<TextOp>),
    Normal(String),
}

impl From<(TextToken, &Config)> for TextOp {
    fn from((value, config): (TextToken, &Config)) -> Self {
        match value {
            TextToken::Verbatim(tokens) => Self::Verbatim(
                tokens
                    .into_iter()
                    .map(|token| Self::from((token, config)))
                    .collect(),
            ),
            TextToken::Underline(tokens) => Self::Underline(
                tokens
                    .into_iter()
                    .map(|token| Self::from((token, config)))
                    .collect(),
            ),
            TextToken::Crossed(tokens) => Self::Crossed(
                tokens
                    .into_iter()
                    .map(|token| Self::from((token, config)))
                    .collect(),
            ),
            TextToken::Bold(tokens) => Self::Bold(
                tokens
                    .into_iter()
                    .map(|token| Self::from((token, config)))
                    .collect(),
            ),
            TextToken::Italic(tokens) => Self::Italic(
                tokens
                    .into_iter()
                    .map(|token| Self::from((token, config)))
                    .collect(),
            ),
            TextToken::Link {
                name,
                handler,
                path,
            } => Self::Link {
                name,
                handler: Handler::from((handler, config)),
                path,
            },
            TextToken::TextExtra(char, tokens) => Self::TextExtra(
                char,
                tokens
                    .into_iter()
                    .map(|token| Self::from((token, config)))
                    .collect(),
            ),
            TextToken::Text(str) => Self::Normal(str),
        }
    }
}

impl std::fmt::Display for TextOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Verbatim(strs) => format!(
                    "`{}`",
                    strs.into_iter()
                        .map(|str| str.to_string())
                        .collect::<Vec<String>>()
                        .join("")
                ),
                Self::Underline(strs) => format!(
                    "_{}_",
                    strs.into_iter()
                        .map(|str| str.to_string())
                        .collect::<Vec<String>>()
                        .join("")
                ),
                Self::Crossed(strs) => format!(
                    "-{}-",
                    strs.into_iter()
                        .map(|str| str.to_string())
                        .collect::<Vec<String>>()
                        .join("")
                ),
                Self::Bold(strs) => format!(
                    "*{}*",
                    strs.into_iter()
                        .map(|str| str.to_string())
                        .collect::<Vec<String>>()
                        .join("")
                ),
                Self::Italic(strs) => format!(
                    "/{}/",
                    strs.into_iter()
                        .map(|str| str.to_string())
                        .collect::<Vec<String>>()
                        .join("")
                ),
                Self::Link {
                    name,
                    handler,
                    path,
                } => format!("|{name}|"),
                Self::TextExtra(char, strs) => {
                    format!(
                        "{char}{}",
                        strs.into_iter()
                            .map(|str| str.to_string())
                            .collect::<Vec<String>>()
                            .join("")
                    )
                }
                Self::Normal(str) => str.to_owned(),
            }
        )
    }
}
