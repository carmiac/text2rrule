use crate::en;
use crate::eo;
use crate::{error::ParseError, token::Token};

/// The parsers for each implemented locale.
pub enum Parser {
    En, // English
    Eo, // Esperanto
}

impl Parser {
    /// Finds the best parser for the priority list of locales.
    pub fn get_parser(mut locales: impl Iterator<Item = String>) -> Option<Parser> {
        locales.find_map(
            |locale| match locale.split_once('-').map(|(l, _)| l).unwrap_or(&locale) {
                "en" => Some(Parser::En),
                "eo" => Some(Parser::Eo),
                _ => None,
            },
        )
    }
    pub fn normalize(&self, input: &str) -> String {
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
