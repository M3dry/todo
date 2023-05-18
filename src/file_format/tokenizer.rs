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
    Verbatim(Vec<TextToken>),
    Underline(Vec<TextToken>),
    Crossed(Vec<TextToken>),
    Bold(Vec<TextToken>),
    Italic(Vec<TextToken>),
    TextExtra(char, Vec<TextToken>),
    Text(String),
}

impl TextToken {
    fn from_iter<I: Iterator<Item = char>>(iter: &mut Peekable<I>) -> Self {
        match iter.peek().unwrap() {
            '\n' => return Self::Text(format!("")),
            '`' => {
                iter.next();
                let mut ret = vec![Self::from_iter(iter)];

                while let Some(char) = iter.peek() {
                    if *char == '\n' {
                        return Self::TextExtra('`', ret);
                    } else if *char == '`' {
                        iter.next();
                        break;
                    } else if ['_', '-', '*', '/'].contains(char) {
                        let ch = *char;
                        let token = Self::from_iter(iter);

                        if matches!(&token, Self::Text(text) if text.is_empty()) {
                            ret.push(Self::TextExtra(ch, vec![]))
                        }
                        ret.push(token);
                    }
                }

                return Self::Verbatim(ret);
            },
            '_' => {
                iter.next();
                let mut ret = vec![Self::from_iter(iter)];

                while let Some(char) = iter.peek() {
                    if *char == '\n' {
                        return Self::TextExtra('_', ret);
                    } else if *char == '_' {
                        iter.next();
                        break;
                    } else if ['`', '-', '*', '/'].contains(char) {
                        ret.push(Self::from_iter(iter));
                    }
                }

                return Self::Underline(ret);
            },
            '-' => {
                iter.next();
                let mut ret = vec![Self::from_iter(iter)];

                while let Some(char) = iter.peek() {
                    if *char == '\n' {
                        return Self::TextExtra('-', ret);
                    } else if *char == '-' {
                        iter.next();
                        break;
                    } else if ['`', '_', '*', '/'].contains(char) {
                        ret.push(Self::from_iter(iter));
                    }
                }

                return Self::Crossed(ret);
            },
            '*' => {
                iter.next();
                let mut ret = vec![Self::from_iter(iter)];

                while let Some(char) = iter.peek() {
                    if *char == '\n' {
                        return Self::TextExtra('*', ret);
                    } else if *char == '*' {
                        iter.next();
                        break;
                    } else if ['`', '_', '-', '/'].contains(char) {
                        ret.push(Self::from_iter(iter));
                    }
                }

                return Self::Bold(ret);
            },
            '/' => {
                iter.next();
                let mut ret = vec![Self::from_iter(iter)];

                while let Some(char) = iter.peek() {
                    if *char == '\n' {
                        return Self::TextExtra('/', ret);
                    } else if *char == '/' {
                        iter.next();
                        break;
                    } else if ['`', '_', '-', '*'].contains(char) {
                        ret.push(Self::from_iter(iter));
                    }
                }

                return Self::Italic(ret);
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
