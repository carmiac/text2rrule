mod en;
mod eo;
pub mod error;
mod parser;
mod token;
pub use error::ParseError;

/// Takes a str reference and attempts to convert it into a rrule.
///
/// Uses the current locale, with fallback to English if it can't be determined
/// or is not supported.
pub fn text2rrule(input: &str) -> Result<String, ParseError> {
    let parser = parser::get_parser();
    let normalized = parser.normalize(input)?;
    let tokens = parser.tokenize(&normalized)?;
    // let pattern = pattern::patternize(&tokens)?;
    // return emit::rrule(&pattern)?;
    todo!()
}

#[cfg(test)]
mod tests {}
