use std::{str::FromStr, collections::VecDeque, iter::Peekable};

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
        let mut chars = s.chars().peekable();

        while let Some(char) = chars.peek() {
            match char {
                '[' => {
                    chars.next();
                    tokens.push_back(Token::BracketOpen);
                    let mut inside = vec![];

                    while let Some(' ') = chars.peek() {
                        chars.next();
                    }
                    
                    while let Some(char) = chars.next() {
                        if char == ']' {
                            break;
                        }

                        inside.push(char);
                    }

                    tokens.push_back(Token::Inside(inside.into_iter().collect()));
                    tokens.push_back(Token::BracketClose);
                },
                '#' => {
                    chars.next();
                    let mut heading = vec![];

                    while let Some(' ') = chars.peek() {
                        chars.next();
                    }

                    while let Some(char) = chars.next() {
                        if char == '\n' {
                            tokens.push_back(Token::Heading(heading.into_iter().collect()));
                            tokens.push_back(Token::Newline);
                            break;
                        }

                        heading.push(char);
                    }
                }
                '\n' => {
                    chars.next();
                    tokens.push_back(Token::Newline)
                },
                '-' => {
                    chars.next();
                    while let Some(' ') = chars.peek() {
                        chars.next();
                    }
                    
                    tokens.push_back(Token::Bullet(TextTokens::from_iter(&mut chars)))
                },
                ' ' => {
                    chars.next();
                },
                _ => {
                    while let Some(' ') = chars.peek() {
                        chars.next();
                    }
                    
                    tokens.push_back(Token::Text(TextTokens::from_iter(&mut chars)))
                }
            }
        }

        return Ok(Self(tokens));
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextToken {
    Verbatim(Box<TextToken>),
    Underline(Box<TextToken>),
    Crossed(Box<TextToken>),
    Bold(Box<TextToken>),
    Italic(Box<TextToken>),
    Text(String),
}

impl TextToken {
    fn from_iter<I: Iterator<Item = char>>(iter: &mut Peekable<I>) -> Self {
        match iter.peek().unwrap() {
            '\n' => unreachable!(),
            '`' => {
                iter.next();
                let ret = Self::Verbatim(Box::new(Self::from_iter(iter)));

                if iter.next() != Some('`') {
                    todo!()
                }

                return ret;
            },
            '_' => {
                iter.next();
                let ret = Self::Underline(Box::new(Self::from_iter(iter)));

                if iter.next() != Some('_') {
                    todo!()
                }

                return ret;
            },
            '-' => {
                iter.next();
                let ret = Self::Crossed(Box::new(Self::from_iter(iter)));

                if iter.next() != Some('-') {
                    todo!()
                }

                return ret;
            },
            '*' => {
                iter.next();
                let ret = Self::Bold(Box::new(Self::from_iter(iter)));

                if iter.next() != Some('*') {
                    todo!()
                }

                return ret;
            },
            '/' => {
                iter.next();
                let ret = Self::Italic(Box::new(Self::from_iter(iter)));

                if iter.next() != Some('/') {
                    todo!()
                }

                return ret;
            },
            _ => {
                let mut text = vec![iter.next().unwrap()];
                while let Some(char) = iter.peek() {
                    if ['`', '_', '-', '*', '/', '\n'].contains(char) {
                        break;
                    }

                    text.push(iter.next().unwrap())
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

    fn from_iter<I: Iterator<Item = char>>(iter: &mut Peekable<I>) -> Self {
        let mut tokens = VecDeque::new();

        while let Some(char) = iter.peek() {
            match char {
                '\n' => return Self(tokens),
                _ => tokens.push_back(TextToken::from_iter(iter))
            }
        }

        return Self(tokens);
    }
}
