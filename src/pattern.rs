//! Pattern matching for tokens.
use std::collections::HashSet;

use crate::error::ParseError;
use crate::token::{DaySet, FreqWord, Month, Token, TokenTag, Weekday};
use chrono::{NaiveDate, NaiveTime};
use tracing::debug;

/// Optional modifiers that can attach to any RecurrencePattern.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Modifiers {
    pub until: Option<NaiveDate>,
    pub count: Option<u32>,
    pub time: Option<NaiveTime>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RecurrencePattern {
    // "every 2 weeks"
    Simple {
        freq: FreqWord,
        interval: u32,
    },

    // "every monday and wednesday"
    ByWeekday {
        days: Vec<Weekday>,
        interval: u32,
    },

    // "every weekday", "every weekend"
    WeekdaySet {
        set: DaySet,
        interval: u32,
    },

    // "monthly on the 15th", "the 1st and 15th of every month"
    MonthlyByDay {
        interval: u32,
        days: Vec<u8>,
    },

    // "monthly on the third tuesday", "the first and third monday"
    MonthlyByPosition {
        interval: u32,
        days: Vec<(i8, Weekday)>,
    },

    // "every year on june 1st", "every year on jan 1 and july 1"
    YearlyByMonth {
        month: Month,
        days: Vec<u8>,
    },

    // "the last monday of may", "the first and third monday of may"
    YearlyByPosition {
        month: Month,
        days: Vec<(i8, Weekday)>,
    },
}

fn first_freq(tokens: &[Token]) -> Option<FreqWord> {
    tokens.iter().find_map(|t| {
        if let Token::Frequency(f) = t {
            Some(*f)
        } else {
            None
        }
    })
}

fn first_month(tokens: &[Token]) -> Option<Month> {
    tokens.iter().find_map(|t| {
        if let Token::Month(m) = t {
            Some(*m)
        } else {
            None
        }
    })
}

fn first_weekday_set(tokens: &[Token]) -> Option<DaySet> {
    tokens.iter().find_map(|t| {
        if let Token::WeekdaySet(s) = t {
            Some(*s)
        } else {
            None
        }
    })
}

/// Pairs each weekday with the ordinal positions right before it.
///
/// "1 3 monday 2 friday" -> [(1, monday), (3, monday), (2, friday)]. A position without a
/// following weekday, or a weekday without a position, is an error.
fn position_pairs(tokens: &[Token]) -> Result<Vec<(i8, Weekday)>, ParseError> {
    let unpaired =
        || ParseError::UnsupportedPattern(format!("unpaired position or weekday: {:?}", tokens));
    let mut pairs = Vec::new();
    let mut pending: Vec<i8> = Vec::new();
    for token in tokens {
        match token {
            Token::OrdinalPosition(p) => pending.push(*p as i8),
            Token::Weekday(w) => {
                if pending.is_empty() {
                    return Err(unpaired());
                }
                pairs.extend(pending.drain(..).map(|p| (p, *w)));
            }
            _ => {}
        }
    }
    if !pending.is_empty() {
        return Err(unpaired());
    }
    Ok(pairs)
}

fn interval_or_1(tokens: &[Token]) -> u32 {
    tokens
        .iter()
        .find_map(|t| {
            if let Token::Interval(n) = t {
                Some(*n)
            } else {
                None
            }
        })
        .unwrap_or(1)
}

fn all_monthdays(tokens: &[Token]) -> Vec<u8> {
    tokens
        .iter()
        .filter_map(|t| {
            if let Token::MonthDay(d) = t {
                Some(*d)
            } else {
                None
            }
        })
        .collect()
}

fn all_weekdays(tokens: &[Token]) -> Vec<Weekday> {
    tokens
        .iter()
        .filter_map(|t| {
            if let Token::Weekday(w) = t {
                Some(*w)
            } else {
                None
            }
        })
        .collect()
}

/// Takes a Vec<Token> produced by tokenize() and converts it into a RecurrencePattern.
///
/// Any modifier tokens (UntilDate, Count, TimeOfDay) are extracted into a Modifiers struct.
pub fn patternize(tokens: Vec<Token>) -> Result<(RecurrencePattern, Modifiers), ParseError> {
    // Pull modifier tokens out into the Modifiers struct.
    let mut modifiers = Modifiers::default();
    let tokens: Vec<Token> = tokens
        .into_iter()
        .filter(|t| match t {
            Token::UntilDate(d) => {
                modifiers.until = Some(*d);
                false
            }
            Token::Count(n) => {
                modifiers.count = Some(*n);
                false
            }
            Token::TimeOfDay(t) => {
                modifiers.time = Some(*t);
                false
            }
            _ => true,
        })
        .collect();
    debug!("modifiers: {:?}", modifiers);
    debug!("tokens: {:?}", tokens);

    // To determine the recurrence type, first determine the types of tokens.
    // Each recurrence type has a different set of token.
    let mut tags = HashSet::new();
    for token in &tokens {
        tags.insert(token.tag());
    }

    let required = |ts: &[TokenTag]| ts.iter().all(|t| tags.contains(t));
    let excluded = |ts: &[TokenTag]| ts.iter().all(|t| !tags.contains(t));

    let missing =
        |what: &str| ParseError::UnsupportedPattern(format!("missing {}: {:?}", what, tokens));

    // Simple -> Freqency and optionaly an Interval and TimeOfDay
    if required(&[TokenTag::Frequency])
        && excluded(&[
            TokenTag::Weekday,
            TokenTag::WeekdaySet,
            TokenTag::MonthDay,
            TokenTag::Month,
            TokenTag::OrdinalPosition,
        ])
    {
        let freq = first_freq(&tokens).ok_or_else(|| missing("frequency"))?;
        let interval = interval_or_1(&tokens);
        return Ok((RecurrencePattern::Simple { freq, interval }, modifiers));
    }

    // WeekdaySet -> "every weekday" / "every weekend"
    if required(&[TokenTag::WeekdaySet])
        && excluded(&[
            TokenTag::Weekday,
            TokenTag::MonthDay,
            TokenTag::Month,
            TokenTag::OrdinalPosition,
        ])
    {
        let set = first_weekday_set(&tokens).ok_or_else(|| missing("weekday set"))?;
        let interval = interval_or_1(&tokens);
        return Ok((RecurrencePattern::WeekdaySet { set, interval }, modifiers));
    }

    // ByWeekday -> one or more Weekday tokens, optional Frequency and Interval
    if required(&[TokenTag::Weekday])
        && excluded(&[
            TokenTag::WeekdaySet,
            TokenTag::MonthDay,
            TokenTag::Month,
            TokenTag::OrdinalPosition,
        ])
    {
        let days = all_weekdays(&tokens);
        let interval = interval_or_1(&tokens);
        return Ok((RecurrencePattern::ByWeekday { days, interval }, modifiers));
    }

    // MonthlyByDay -> "monthly on the Nth", requires MonthDay and Frequency
    if required(&[TokenTag::MonthDay, TokenTag::Frequency])
        && excluded(&[
            TokenTag::Weekday,
            TokenTag::WeekdaySet,
            TokenTag::Month,
            TokenTag::OrdinalPosition,
        ])
    {
        let days = all_monthdays(&tokens);
        let interval = interval_or_1(&tokens);
        return Ok((
            RecurrencePattern::MonthlyByDay { interval, days },
            modifiers,
        ));
    }

    // YearlyByMonth -> "every year on the Nth of <month>"
    if required(&[TokenTag::Month, TokenTag::Frequency, TokenTag::MonthDay])
        && excluded(&[
            TokenTag::Weekday,
            TokenTag::WeekdaySet,
            TokenTag::OrdinalPosition,
        ])
    {
        let month = first_month(&tokens).ok_or_else(|| missing("month"))?;
        let days = all_monthdays(&tokens);
        return Ok((RecurrencePattern::YearlyByMonth { month, days }, modifiers));
    }

    // YearlyByPosition -> "the last monday of may", "the first and third monday of may"
    // Frequency is optional, but must be yearly if given.
    if required(&[
        TokenTag::Month,
        TokenTag::OrdinalPosition,
        TokenTag::Weekday,
    ]) && excluded(&[TokenTag::WeekdaySet, TokenTag::MonthDay])
        && matches!(first_freq(&tokens), None | Some(FreqWord::Yearly))
    {
        let month = first_month(&tokens).ok_or_else(|| missing("month"))?;
        let days = position_pairs(&tokens)?;
        return Ok((
            RecurrencePattern::YearlyByPosition { month, days },
            modifiers,
        ));
    }

    // MonthlyByPosition -> "monthly on the third tuesday", "first and third monday"
    // Frequency is optional, but must be monthly if given.
    if required(&[TokenTag::OrdinalPosition, TokenTag::Weekday])
        && excluded(&[TokenTag::WeekdaySet, TokenTag::MonthDay, TokenTag::Month])
        && matches!(first_freq(&tokens), None | Some(FreqWord::Monthly))
    {
        let days = position_pairs(&tokens)?;
        let interval = interval_or_1(&tokens);
        return Ok((
            RecurrencePattern::MonthlyByPosition { interval, days },
            modifiers,
        ));
    }

    Err(ParseError::UnsupportedPattern(format!(
        "Unknown pattern: {:?}",
        tokens,
    )))
}

#[cfg(test)]
mod tests {
    use crate::pattern::RecurrencePattern::{
        ByWeekday, MonthlyByDay, MonthlyByPosition, Simple, WeekdaySet, YearlyByMonth,
        YearlyByPosition,
    };
    use crate::pattern::{Modifiers, patternize};
    use crate::token::DaySet::*;
    use crate::token::FreqWord::*;
    use crate::token::Month::*;
    use crate::token::Token;
    use crate::token::Weekday::*;
    use Token::*;

    #[test]
    fn simple() {
        assert_eq!(
            patternize(vec![Frequency(Daily)]),
            Ok((
                Simple {
                    freq: Daily,
                    interval: 1
                },
                Modifiers::default()
            ))
        );
        assert_eq!(
            patternize(vec![Token::Frequency(Weekly), Count(3)]),
            Ok((
                Simple {
                    freq: Weekly,
                    interval: 1
                },
                Modifiers {
                    count: Some(3),
                    ..Modifiers::default()
                }
            ))
        );
        assert_eq!(
            patternize(vec![Frequency(Monthly), Interval(2)]),
            Ok((
                Simple {
                    freq: Monthly,
                    interval: 2
                },
                Modifiers::default()
            ))
        );
        assert_eq!(
            patternize(vec![Interval(3), Frequency(Yearly)]),
            Ok((
                Simple {
                    freq: Yearly,
                    interval: 3
                },
                Modifiers::default()
            ))
        );
    }

    #[test]
    fn by_weekday() {
        assert_eq!(
            patternize(vec![Weekday(Monday)]),
            Ok((
                ByWeekday {
                    days: vec![Monday],
                    interval: 1
                },
                Modifiers::default()
            ))
        );
        assert_eq!(
            patternize(vec![Frequency(Weekly), Weekday(Monday)]),
            Ok((
                ByWeekday {
                    days: vec![Monday],
                    interval: 1
                },
                Modifiers::default()
            ))
        );
        assert_eq!(
            patternize(vec![Interval(2), Weekday(Tuesday)]),
            Ok((
                ByWeekday {
                    days: vec![Tuesday],
                    interval: 2
                },
                Modifiers::default()
            ))
        );
        assert_eq!(
            patternize(vec![Weekday(Monday), Weekday(Wednesday), Weekday(Friday)]),
            Ok((
                ByWeekday {
                    days: vec![Monday, Wednesday, Friday],
                    interval: 1
                },
                Modifiers::default()
            ))
        );
        assert_eq!(
            patternize(vec![Frequency(Weekly), Weekday(Monday), Count(3)]),
            Ok((
                ByWeekday {
                    days: vec![Monday],
                    interval: 1
                },
                Modifiers {
                    count: Some(3),
                    ..Modifiers::default()
                }
            ))
        );
    }

    #[test]
    fn yearly_by_month() {
        assert_eq!(
            patternize(vec![Frequency(Yearly), MonthDay(1), Month(June)]),
            Ok((
                YearlyByMonth {
                    month: June,
                    days: vec![1]
                },
                Modifiers::default()
            ))
        );
        assert_eq!(
            patternize(vec![Frequency(Yearly), Month(March), MonthDay(15)]),
            Ok((
                YearlyByMonth {
                    month: March,
                    days: vec![15]
                },
                Modifiers::default()
            ))
        );
        assert_eq!(
            patternize(vec![
                Frequency(Yearly),
                Month(January),
                MonthDay(1),
                MonthDay(15)
            ]),
            Ok((
                YearlyByMonth {
                    month: January,
                    days: vec![1, 15]
                },
                Modifiers::default()
            ))
        );
    }

    #[test]
    fn yearly_by_position() {
        // Memorial Day: last monday of may
        assert_eq!(
            patternize(vec![
                Frequency(Yearly),
                OrdinalPosition(-1),
                Weekday(Monday),
                Month(May)
            ]),
            Ok((
                YearlyByPosition {
                    month: May,
                    days: vec![(-1, Monday)]
                },
                Modifiers::default()
            ))
        );
        // Thanksgiving: fourth thursday of november
        assert_eq!(
            patternize(vec![
                Frequency(Yearly),
                Month(November),
                OrdinalPosition(4),
                Weekday(Thursday)
            ]),
            Ok((
                YearlyByPosition {
                    month: November,
                    days: vec![(4, Thursday)]
                },
                Modifiers::default()
            ))
        );
    }

    #[test]
    fn monthly_by_position() {
        assert_eq!(
            patternize(vec![
                Frequency(Monthly),
                OrdinalPosition(3),
                Weekday(Friday)
            ]),
            Ok((
                MonthlyByPosition {
                    interval: 1,
                    days: vec![(3, Friday)]
                },
                Modifiers::default()
            ))
        );
        assert_eq!(
            patternize(vec![
                OrdinalPosition(-1),
                Weekday(Friday),
                Frequency(Monthly)
            ]),
            Ok((
                MonthlyByPosition {
                    interval: 1,
                    days: vec![(-1, Friday)]
                },
                Modifiers::default()
            ))
        );
        assert_eq!(
            patternize(vec![
                OrdinalPosition(2),
                Weekday(Wednesday),
                Frequency(Monthly)
            ]),
            Ok((
                MonthlyByPosition {
                    interval: 1,
                    days: vec![(2, Wednesday)]
                },
                Modifiers::default()
            ))
        );
    }

    #[test]
    fn multi_position() {
        // "monthly on the first monday and third wednesday"
        assert_eq!(
            patternize(vec![
                Frequency(Monthly),
                OrdinalPosition(1),
                Weekday(Monday),
                OrdinalPosition(3),
                Weekday(Wednesday)
            ]),
            Ok((
                MonthlyByPosition {
                    interval: 1,
                    days: vec![(1, Monday), (3, Wednesday)]
                },
                Modifiers::default()
            ))
        );
        // "the first and third monday of may"
        assert_eq!(
            patternize(vec![
                OrdinalPosition(1),
                OrdinalPosition(3),
                Weekday(Monday),
                Month(May)
            ]),
            Ok((
                YearlyByPosition {
                    month: May,
                    days: vec![(1, Monday), (3, Monday)]
                },
                Modifiers::default()
            ))
        );
    }

    #[test]
    fn position_without_frequency() {
        // "first and third monday" -> monthly
        assert_eq!(
            patternize(vec![
                OrdinalPosition(1),
                OrdinalPosition(3),
                Weekday(Monday)
            ]),
            Ok((
                MonthlyByPosition {
                    interval: 1,
                    days: vec![(1, Monday), (3, Monday)]
                },
                Modifiers::default()
            ))
        );
        // "the first monday of may" -> yearly
        assert_eq!(
            patternize(vec![OrdinalPosition(1), Weekday(Monday), Month(May)]),
            Ok((
                YearlyByPosition {
                    month: May,
                    days: vec![(1, Monday)]
                },
                Modifiers::default()
            ))
        );
    }

    #[test]
    fn position_errors() {
        // wrong frequency
        assert!(patternize(vec![Frequency(Weekly), OrdinalPosition(1), Weekday(Monday)]).is_err());
        assert!(
            patternize(vec![
                Frequency(Monthly),
                OrdinalPosition(1),
                Weekday(Monday),
                Month(May)
            ])
            .is_err()
        );
        // position with no weekday after it
        assert!(
            patternize(vec![
                OrdinalPosition(1),
                Weekday(Monday),
                OrdinalPosition(3)
            ])
            .is_err()
        );
        // weekday with no position before it
        assert!(
            patternize(vec![
                OrdinalPosition(1),
                Weekday(Monday),
                Weekday(Wednesday)
            ])
            .is_err()
        );
    }

    #[test]
    fn monthly_by_day() {
        assert_eq!(
            patternize(vec![Frequency(Monthly), MonthDay(15)]),
            Ok((
                MonthlyByDay {
                    interval: 1,
                    days: vec![15]
                },
                Modifiers::default()
            ))
        );
        assert_eq!(
            patternize(vec![MonthDay(2), Frequency(Monthly)]),
            Ok((
                MonthlyByDay {
                    interval: 1,
                    days: vec![2]
                },
                Modifiers::default()
            ))
        );
        assert_eq!(
            patternize(vec![Interval(3), Frequency(Monthly), MonthDay(4)]),
            Ok((
                MonthlyByDay {
                    interval: 3,
                    days: vec![4]
                },
                Modifiers::default()
            ))
        );
        // "the 1st and 15th of every month"
        assert_eq!(
            patternize(vec![Frequency(Monthly), MonthDay(1), MonthDay(15)]),
            Ok((
                MonthlyByDay {
                    interval: 1,
                    days: vec![1, 15]
                },
                Modifiers::default()
            ))
        );
    }

    #[test]
    fn weekday_set() {
        assert_eq!(
            patternize(vec![Token::WeekdaySet(Weekdays)]),
            Ok((
                WeekdaySet {
                    set: Weekdays,
                    interval: 1
                },
                Modifiers::default()
            ))
        );
        assert_eq!(
            patternize(vec![Token::WeekdaySet(Weekend)]),
            Ok((
                WeekdaySet {
                    set: Weekend,
                    interval: 1
                },
                Modifiers::default()
            ))
        );
        assert_eq!(
            patternize(vec![Interval(2), Token::WeekdaySet(Weekdays)]),
            Ok((
                WeekdaySet {
                    set: Weekdays,
                    interval: 2
                },
                Modifiers::default()
            ))
        );
    }
}
