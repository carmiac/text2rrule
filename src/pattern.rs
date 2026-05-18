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

    // "monthly on the third tuesday"
    MonthlyByPosition {
        interval: u32,
        pos: i8,
        weekday: Weekday,
    },

    // "every year on june 1st", "every year on jan 1 and july 1"
    YearlyByMonth {
        month: Month,
        days: Vec<u8>,
    },

    // "the last monday of may", "the third thursday of november"
    YearlyByPosition {
        month: Month,
        pos: i8,
        weekday: Weekday,
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

fn first_weekday(tokens: &[Token]) -> Option<Weekday> {
    tokens.iter().find_map(|t| {
        if let Token::Weekday(w) = t {
            Some(*w)
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

fn first_position(tokens: &[Token]) -> Option<i8> {
    tokens.iter().find_map(|t| {
        if let Token::OrdinalPosition(p) = t {
            Some(*p as i8)
        } else {
            None
        }
    })
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

    // YearlyByPosition -> "the last monday of may", "the third thursday of november"
    if required(&[
        TokenTag::Month,
        TokenTag::OrdinalPosition,
        TokenTag::Weekday,
        TokenTag::Frequency,
    ]) && excluded(&[TokenTag::WeekdaySet, TokenTag::MonthDay])
    {
        let month = first_month(&tokens).ok_or_else(|| missing("month"))?;
        let pos = first_position(&tokens).ok_or_else(|| missing("ordinal position"))?;
        let weekday = first_weekday(&tokens).ok_or_else(|| missing("weekday"))?;
        return Ok((
            RecurrencePattern::YearlyByPosition {
                month,
                pos,
                weekday,
            },
            modifiers,
        ));
    }

    // MonthlyByPosition -> "monthly on the third tuesday", "every last fri of the month"
    if required(&[
        TokenTag::OrdinalPosition,
        TokenTag::Weekday,
        TokenTag::Frequency,
    ]) && excluded(&[TokenTag::WeekdaySet, TokenTag::MonthDay, TokenTag::Month])
    {
        let pos = first_position(&tokens).ok_or_else(|| missing("ordinal position"))?;
        let weekday = first_weekday(&tokens).ok_or_else(|| missing("weekday"))?;
        let interval = interval_or_1(&tokens);
        return Ok((
            RecurrencePattern::MonthlyByPosition {
                interval,
                pos,
                weekday,
            },
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
                    pos: -1,
                    weekday: Monday
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
                    pos: 4,
                    weekday: Thursday
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
                    pos: 3,
                    weekday: Friday
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
                    pos: -1,
                    weekday: Friday
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
                    pos: 2,
                    weekday: Wednesday
                },
                Modifiers::default()
            ))
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
