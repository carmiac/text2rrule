use crate::en;
use crate::eo;
use crate::{error::ParseError, token::Token};

use sys_locale::get_locales;

/// The parsers for each implemented locale.
pub enum Parser {
    En, // English
    Eo, // Esperanto
}

pub fn get_parser() -> Parser {
    get_locales()
        .find_map(
            |locale| match locale.split_once('-').map(|(l, _)| l).unwrap_or(&locale) {
                "en" => Some(Parser::En),
                "eo" => Some(Parser::Eo),
                _ => None,
            },
        )
        .unwrap_or(Parser::En)
}
impl Parser {
    pub fn normalize(&self, input: &str) -> Result<String, ParseError> {
        match self {
            Parser::En => en::normalize(input),
            Parser::Eo => eo::normalize(input),
        }
    }
    pub fn tokenize(&self, input: &str) -> Result<Vec<Token>, ParseError> {
        match self {
            Parser::En => en::tokenize(input),
            Parser::Eo => eo::tokenize(input),
        }
    }
}
