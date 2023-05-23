use std::{collections::VecDeque, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    BracketOpen,
    Inside(String),
    BracketClose,
    Heading(String),
    Bullet(TextTokens),
    Text(TextTokens),
    Newline,
}

pub struct Tokens(VecDeque<Token>);

impl Tokens {
    pub fn to_vecdeque(self) -> VecDeque<Token> {
        self.0
    }
}

impl FromStr for Tokens {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens = VecDeque::new();
        let mut chars = VecDeque::from_iter(s.chars());
        let mut last = Token::Newline;

        while let Some(char) = chars.get(0) {
            match char {
                '[' if matches!(last, Token::Newline) => {
                    chars.pop_front();
                    tokens.push_back(Token::BracketOpen);
                    let mut inside = vec![];

                    while let Some(' ') = chars.get(0) {
                        chars.pop_front();
                    }

                    while let Some(char) = chars.pop_front() {
                        if char == ']' {
                            break;
                        }

                        inside.push(char);
                    }

                    tokens.push_back(Token::Inside(inside.into_iter().collect()));
                    tokens.push_back(Token::BracketClose);

                    last = Token::BracketClose;
                }
                '#' if matches!(last, Token::Newline) => {
                    chars.pop_front();
                    let mut heading = vec![];

                    while let Some(' ') = chars.get(0) {
                        chars.pop_front();
                    }

                    while let Some(char) = chars.pop_front() {
                        if char == '\n' {
                            tokens.push_back(Token::Heading(heading.into_iter().collect()));
                            tokens.push_back(Token::Newline);
                            break;
                        }

                        heading.push(char);
                    }

                    last = Token::Newline;
                }
                '\n' => {
                    chars.pop_front();
                    tokens.push_back(Token::Newline);
                    last = Token::Newline;
                }
                '-' => {
                    chars.pop_front();
                    while let Some(' ') = chars.get(0) {
                        chars.pop_front();
                    }

                    last = Token::Bullet(TextTokens(VecDeque::new()));
                    tokens.push_back(Token::Bullet(TextTokens::from_vecdeque(&mut chars)));
                }
                ' ' => {
                    chars.pop_front();
                }
                _ => {
                    while let Some(' ') = chars.get(0) {
                        chars.pop_front();
                    }

                    last = Token::Text(TextTokens(VecDeque::new()));
                    tokens.push_back(Token::Text(TextTokens::from_vecdeque(&mut chars)))
                }
            }
        }

        return Ok(Self(tokens));
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextToken {
    Verbatim(Vec<TextToken>),
    Underline(Vec<TextToken>),
    Crossed(Vec<TextToken>),
    Bold(Vec<TextToken>),
    Italic(Vec<TextToken>),
    Link {
        name: String,
        handler: String,
        path: String,
    },
    TextExtra(char, Vec<TextToken>),
    Text(String),
}

impl TextToken {
    fn from_vecdeque(chars: &mut VecDeque<char>) -> Self {
        match chars.get(0).unwrap() {
            '\n' => return Self::Text(format!("")),
            '`' => {
                chars.pop_front();
                let mut ret = vec![Self::from_vecdeque(chars)];

                while let Some(char) = chars.get(0) {
                    if *char == '\n' {
                        return Self::TextExtra('`', ret);
                    } else if *char == '`' {
                        chars.pop_front();
                        break;
                    } else if ['_', '-', '*', '/'].contains(char) {
                        let ch = *char;
                        let token = Self::from_vecdeque(chars);

                        if matches!(&token, Self::Text(text) if text.is_empty()) {
                            ret.push(Self::TextExtra(ch, vec![]))
                        }
                        ret.push(token);
                    }
                }

                return Self::Verbatim(ret);
            }
            '_' => {
                chars.pop_front();
                let mut ret = vec![Self::from_vecdeque(chars)];

                while let Some(char) = chars.get(0) {
                    if *char == '\n' {
                        return Self::TextExtra('_', ret);
                    } else if *char == '_' {
                        chars.pop_front();
                        break;
                    } else if ['`', '-', '*', '/'].contains(char) {
                        ret.push(Self::from_vecdeque(chars));
                    }
                }

                return Self::Underline(ret);
            }
            '-' => {
                chars.pop_front();
                let mut ret = vec![Self::from_vecdeque(chars)];

                while let Some(char) = chars.get(0) {
                    if *char == '\n' {
                        return Self::TextExtra('-', ret);
                    } else if *char == '-' {
                        chars.pop_front();
                        break;
                    } else if ['`', '_', '*', '/'].contains(char) {
                        ret.push(Self::from_vecdeque(chars));
                    }
                }

                return Self::Crossed(ret);
            }
            '*' => {
                chars.pop_front();
                let mut ret = vec![Self::from_vecdeque(chars)];

                while let Some(char) = chars.get(0) {
                    if *char == '\n' {
                        return Self::TextExtra('*', ret);
                    } else if *char == '*' {
                        chars.pop_front();
                        break;
                    } else if ['`', '_', '-', '/'].contains(char) {
                        ret.push(Self::from_vecdeque(chars));
                    }
                }

                return Self::Bold(ret);
            }
            '/' => {
                chars.pop_front();
                let mut ret = vec![Self::from_vecdeque(chars)];

                while let Some(char) = chars.get(0) {
                    if *char == '\n' {
                        return Self::TextExtra('/', ret);
                    } else if *char == '/' {
                        chars.pop_front();
                        break;
                    } else if ['`', '_', '-', '*'].contains(char) {
                        ret.push(Self::from_vecdeque(chars));
                    }
                }

                return Self::Italic(ret);
            }
            '|' => {
                chars.pop_front();

                for ch in &*chars {
                    if *ch == '|' {
                        return Self::Link { name: todo!(), handler: todo!(), path: todo!() };
                    } else if *ch == '\n' {
                        return Self::TextExtra('|', vec![Self::from_vecdeque(chars)]);
                    }
                }

                todo!()
            },
            _ => {
                let mut text = vec![chars.pop_front().unwrap()];
                while let Some(char) = chars.get(0) {
                    if ['`', '_', '-', '*', '/', '\n'].contains(char) {
                        break;
                    }

                    text.push(chars.pop_front().unwrap())
                }

                return Self::Text(text.into_iter().collect());
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextTokens(VecDeque<TextToken>);

impl TextTokens {
    pub fn to_vecdeque(self) -> VecDeque<TextToken> {
        self.0
    }

    fn from_vecdeque(chars: &mut VecDeque<char>) -> Self {
        let mut tokens = VecDeque::new();

        while let Some(char) = chars.get(0) {
            match char {
                '\n' => return Self(tokens),
                _ => tokens.push_back(TextToken::from_vecdeque(chars)),
            }
        }

        return Self(tokens);
    }
}
