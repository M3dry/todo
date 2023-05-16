use crate::file_format::tokenizer::Token;

#[derive(Debug)]
pub enum Error {
    NoTokens,
    Other(String),
    ExpectedV(Vec<&'static str>, Token),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoTokens => write!(f, "Expected more tokens"),
            Error::Other(str) => write!(f, "{str}"),
            Error::ExpectedV(expected, got) => write!(f, "Expected {expected:#?},\ngot {got:?}"),
        }
    }
}

#[derive(Debug)]
pub struct ParserError {
    stack: Vec<ParserErrorStack>,
    err: Error,
}

impl ParserError {
    pub(crate) fn new(stack: Vec<ParserErrorStack>, err: Error) -> Self {
        Self { stack, err }
    }

    pub(crate) fn push(&mut self, err: ParserErrorStack) {
        self.stack.push(err);
    }
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:\n{}",
            self.err,
            self.stack
                .iter()
                .map(|stack| format!("{stack}"))
                .collect::<Vec<String>>()
                .join("\n")
        )?;

        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct ParserErrorStack {
    name: &'static str,
    file: &'static str,
    location: (u32, u32),
}

impl ParserErrorStack {
    pub(crate) fn new(name: &'static str, file: &'static str, location: (u32, u32)) -> Self {
        Self {
            name,
            file,
            location,
        }
    }
}

impl std::fmt::Display for ParserErrorStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}({}[{}:{}])",
            self.name, self.file, self.location.0, self.location.1
        )
    }
}

macro_rules! error {
    ($func:literal, $value:ident) => {
        if let Token::Text(iden) = error!($func, $value.pop_front(), [Token::Text(_)])? {
            iden
        } else {
            unreachable!()
        }
    };
    ($error:expr, $name:literal) => {
        $error.map_err(|mut err| {
            err.push(ParserErrorStack::new(
                $name,
                file!(),
                (line!(), column!()),
            ));
            err
        })
    };
    ($initial:expr, $err:expr$(,)?) => {
        ParserError::new(
            vec![ParserErrorStack::new(
                $initial,
                file!(),
                (line!(), column!()),
            )],
            $err,
        )
    };
    ($func:literal, $val:expr, [$($pat:pat_param),+]) => {
        if let Some(res) = $val {
            if matches!(res,  $( $pat )|+) {
                Ok(res)
            } else {
                Err(error!($func, Error::ExpectedV(vec!($( stringify!($pat) ),+), res.to_owned())))
            }
        } else {
            Err(error!($func, Error::NoTokens))
        }
    };
}
