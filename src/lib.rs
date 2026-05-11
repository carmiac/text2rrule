mod en;
mod eo;
pub mod error;
mod parser;
mod token;
pub use error::ParseError;
use sys_locale::get_locales;

use crate::parser::Parser;

/// Takes a str reference and attempts to convert it into a rrule.
///
/// Uses the current locale, with fallback to English if it can't be determined
/// or is not supported.
pub fn text2rrule(input: &str) -> Result<String, ParseError> {
    let locales = get_locales();
    text2rrule_with_locale(input, locales)
}

/// Takes a str reference and converts it to an rrule using the given list of locales.
///
/// Attempts to find the best locale match, with fallback to English if it can't be determined.
pub fn text2rrule_with_locale(
    input: &str,
    locales: impl Iterator<Item = String>,
) -> Result<String, ParseError> {
    let parser = Parser::get_parser(locales).unwrap_or(Parser::En);
    let normalized = parser.normalize(input);
    let tokens = parser.tokenize(&normalized)?;
    // let pattern = pattern::patternize(&tokens)?;
    // return emit::rrule(&pattern)?;
    todo!()
}

#[cfg(test)]
mod tests {}
