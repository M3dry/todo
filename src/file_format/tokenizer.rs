use std::{str::FromStr, collections::VecDeque};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    BracketOpen,
    BracketClose,
    Dash,
    Colon,
    Newline,
    Text(String),
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

        while let Some(char) = chars.next() {
            match char {
                '[' => tokens.push_back(Token::BracketOpen),
                ']' => tokens.push_back(Token::BracketClose),
                ':' => tokens.push_back(Token::Colon),
                '\n' => tokens.push_back(Token::Newline),
                '-' => tokens.push_back(Token::Dash),
                ' ' => continue,
                text => {
                    let mut text = vec![text];

                    while let Some(char) = chars.peek() {
                        if *char != '[' && *char != ']' && *char != '\n' && *char != ':' {
                            text.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }

                    tokens.push_back(Token::Text(text.into_iter().collect()))
                }
            }
        }

        return Ok(Self(tokens));
    }
}
