mod en;
mod eo;
pub mod error;
mod parser;
mod token;
use crate::parser::Parser;
pub use error::ParseError;
use sys_locale::get_locales;
use tracing::debug;

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
    debug!("Input: {:?}", input);
    let locales: Vec<String> = locales.collect();
    debug!("Locales: {:?}", locales);
    let parser = Parser::get_parser(locales.into_iter()).unwrap_or(Parser::En);
    debug!("Parser: {:?}", parser);

    let normalized = parser.normalize(input);
    debug!("Normalized: {:?}", normalized);
    let tokens = parser.tokenize(&normalized)?;
    debug!("Tokens: {:?}", tokens);
    // let pattern = pattern::patternize(&tokens)?;
    // debug!("Pattern: {:?}", pattern);
    // let rrule = emit::rrule(&pattern)?;
    // debug!("RRULE: {:?}", rrule);
    // rrule
    todo!()
}

#[cfg(test)]
mod tests {}
