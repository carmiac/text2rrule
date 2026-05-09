use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnrecognizedInput(String),
    UnsupportedPattern(String),
    AmbiguousInput(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnrecognizedInput(msg) => write!(f, "Unrecognized input: {}", msg),
            ParseError::UnsupportedPattern(msg) => write!(f, "Unsupported pattern: {}", msg),
            ParseError::AmbiguousInput(msg) => write!(f, "Ambiguous input: {}", msg),
        }
    }
}

impl std::error::Error for ParseError {}
