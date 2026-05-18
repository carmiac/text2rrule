use crate::locales::en;
use crate::locales::eo;
use crate::{error::ParseError, token::Token};

/// The parsers for each implemented locale.
#[derive(Debug)]
pub enum Parser {
    En, // English
    Eo, // Esperanto
}

impl Parser {
    /// Finds the best parser for the priority list of locales.
    pub fn get_parser(locales: impl IntoIterator<Item = String>) -> Option<Parser> {
        locales.into_iter().find_map(|locale| {
            match locale.split_once('-').map(|(l, _)| l).unwrap_or(&locale) {
                "en" => Some(Parser::En),
                "eo" => Some(Parser::Eo),
                _ => None,
            }
        })
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
