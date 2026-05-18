use std::fmt;

/// Errors returned by [`crate::text2rrule`] and [`crate::text2rrule_with_locale`].
#[derive(Debug, PartialEq)]
pub enum ParseError {
    /// Input contains a word or token the tokenizer doesn't know how to interpret.
    UnrecognizedInput(String),
    /// The tokens are individually recognized but their combination doesn't map
    /// to any supported recurrence pattern.
    UnsupportedPattern(String),
    /// The input could be interpreted multiple ways and the parser can't pick one
    /// without more context.
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
