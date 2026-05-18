/// Normalize a plain-English recurrence string into a canonical form for tokenization.
pub fn normalize(input: &str) -> String {
    // Basic sanitization: Lowercase, punctuation, whitespace.
    let s: String = input
        .to_lowercase()
        .chars()
        .map(|c| if ",.-()".contains(c) { ' ' } else { c })
        .collect();
    let s = s.split_whitespace().collect::<Vec<_>>().join(" ");

    // multi-word shorthands (must come before single-word expansions)
    let s = s.replace("every other", "every 2");
    let s = s.replace("semi annually", "every 6 months");
    let s = s.replace("once a", "every 1");
    let s = s.replace("once per", "every 1");
    // Note: "twice a/per" is not expanded here, it requires day specification and cannot be
    // expressed as a simple RRULE interval. It will produce an UnsupportedPattern
    // error at the pattern-matching stage.

    // Now all of the single word replacements.
    let mut result: Vec<String> = Vec::new();
    for word in s.split_whitespace() {
        // strip ordinal suffixes from numbers (1st -> 1, 2nd -> 2, etc.)
        let word = strip_ordinal_suffix(&word);
        // Taskwarrior shorthands
        // 123ABC -> 123 ABC
        // but not 12:34 -> 12 :34.
        let num_str: String = word.chars().take_while(|c| c.is_ascii_digit()).collect();
        let num_str_len = num_str.len();
        let w = &word[num_str_len..];
        if num_str_len > 0 {
            if w.starts_with(":") {
                // must be a time, so push it and move on to the next word.
                result.push(word.into());
                continue;
            }
            result.push(num_str);
        }
        let w = match w {
            // period shorthands
            "alternate" => "every 2",
            "fortnightly" => "every 2 weeks",
            "fortnight" => "every 2 weeks",
            "quarterly" => "every 3 months",
            "semiannually" => "every 6 months",
            "biweekly" => "every 2 weeks",
            "bimonthly" => "every 2 months",
            "biannually" => "every 2 years",
            "annually" => "yearly",
            "anually" => "yearly",
            "annualy" => "yearly",

            // day abbreviations/misspellings
            "wednsday" => "wednesday",
            "wednseday" => "wednesday",
            "thurs" => "thursday",
            "thur" => "thursday",
            "tues" => "tuesday",
            "mon" => "monday",
            "tue" => "tuesday",
            "wed" => "wednesday",
            "thu" => "thursday",
            "fri" => "friday",
            "sat" => "saturday",
            "sun" => "sunday",

            // frequency-set abbreviations
            "wkdays" => "weekdays",
            "wkday" => "weekdays",
            "wkdy" => "weekdays",
            "wknds" => "weekends",
            "wknd" => "weekends",

            // month abbreviations (3-letter, unambiguous)
            "jan" => "january",
            "feb" => "february",
            "mar" => "march",
            "apr" => "april",
            "jun" => "june",
            "jul" => "july",
            "aug" => "august",
            "sep" => "september",
            "sept" => "september",
            "oct" => "october",
            "nov" => "november",
            "dec" => "december",

            // ordinal words -> digits
            "first" => "1",
            "second" => "2",
            "third" => "3",
            "fourth" => "4",
            "fifth" => "5",
            "sixth" => "6",
            "seventh" => "7",
            "eighth" => "8",
            "ninth" => "9",
            "tenth" => "10",
            "last" => "-1",

            // cardinal number words -> digits
            "one" => "1",
            "two" => "2",
            "three" => "3",
            "four" => "4",
            "five" => "5",
            "six" => "6",
            "seven" => "7",
            "eight" => "8",
            "nine" => "9",
            "ten" => "10",

            // not found, must be ok as is.
            _ => w,
        };
        if !w.is_empty() {
            result.push(w.into());
        }
    }

    result.join(" ")
}

fn strip_ordinal_suffix(w: &str) -> &str {
    let suffixes = ["st", "nd", "rd", "th"];
    for suffix in &suffixes {
        if let Some(stem) = w.strip_suffix(suffix) {
            if !stem.is_empty() && stem.chars().all(|c| c.is_ascii_digit()) {
                return stem;
            }
        }
    }
    w
}

use crate::{error::ParseError, token::Token};
use chrono::{NaiveDate, NaiveTime};
/// Turn an input str into a vector of tokens.
///
/// input should be normalized
pub fn tokenize(input: &str) -> Result<Vec<Token>, ParseError> {
    use crate::token::DaySet::*;
    use crate::token::FreqWord::*;
    use crate::token::Month::*;
    use crate::token::Weekday::*;
    use Token::*;

    let mut tokens: Vec<Token> = Vec::new();

    let mut twice = false;
    let mut the_context = false; // causes next number to be a MonthDay
    let mut month_context = false; // causes next number after a Month to be a MonthDay
    let mut of_context = false; // causes next Frequency to retroactively convert last Interval -> MonthDay
    let mut until = false;
    let mut until_month: Option<u32> = None;
    let mut until_day: Option<u32> = None;

    for word in input.split_whitespace() {
        let token: Option<Token> = match word {
            // Simple frequencies.
            "daily" | "days" | "day" => Some(Frequency(Daily)),
            "weekly" | "weeks" | "week" => Some(Frequency(Weekly)),
            "monthly" | "months" | "month" => Some(Frequency(Monthly)),
            "yearly" | "years" | "year" => Some(Frequency(Yearly)),

            // Days of the week.
            "monday" => Some(Weekday(Monday)),
            "tuesday" => Some(Weekday(Tuesday)),
            "wednesday" => Some(Weekday(Wednesday)),
            "thursday" => Some(Weekday(Thursday)),
            "friday" => Some(Weekday(Friday)),
            "saturday" => Some(Weekday(Saturday)),
            "sunday" => Some(Weekday(Sunday)),
            "weekend" | "weekends" => Some(WeekdaySet(Weekend)),
            "weekday" | "weekdays" => Some(WeekdaySet(Weekdays)),

            // Months
            "january" => Some(Month(January)),
            "february" => Some(Month(February)),
            "march" => Some(Month(March)),
            "april" => Some(Month(April)),
            "may" => Some(Month(May)),
            "june" => Some(Month(June)),
            "july" => Some(Month(July)),
            "august" => Some(Month(August)),
            "september" => Some(Month(September)),
            "october" => Some(Month(October)),
            "november" => Some(Month(November)),
            "december" => Some(Month(December)),

            // filler words
            "every" | "at" | "on" | "a" | "per" | "and" => continue,

            "the" => {
                the_context = true;
                continue;
            }

            "of" => {
                of_context = true;
                continue;
            }

            // plain numbers: negative -> OrdinalPosition, after "the" or a Month -> MonthDay, else -> Interval
            s if let Ok(n) = s.parse::<i32>() => {
                if n < 0 {
                    Some(OrdinalPosition(n))
                } else if the_context {
                    the_context = false;
                    Some(MonthDay(n as u8))
                } else if month_context && !until {
                    month_context = false;
                    Some(MonthDay(n as u8))
                } else {
                    Some(Interval(n as u32))
                }
            }

            "until" => {
                until = true;
                continue;
            }

            // "3 times" -> Count(3): convert the last Interval that was already pushed
            "times" => {
                if let Some(Interval(n)) = tokens.last().copied() {
                    *tokens.last_mut().unwrap() = Count(n as u32);
                }
                continue;
            }

            "twice" => {
                twice = true;
                continue;
            }

            // Time
            s if is_time(s) => {
                if let Ok(t) = NaiveTime::parse_from_str(s, "%H:%M") {
                    Some(TimeOfDay(t))
                } else if let Ok(t) = NaiveTime::parse_from_str(s, "%I:%M%P") {
                    Some(TimeOfDay(t))
                } else {
                    return Err(ParseError::UnrecognizedInput(format!(
                        "unrecognized time: {s}"
                    )));
                }
            }
            _ => {
                return Err(ParseError::UnrecognizedInput(format!(
                    "unrecognized word: {word}"
                )));
            }
        };

        if let Some(token) = token {
            // "twice a year" special case
            if twice {
                if token == Frequency(Yearly) {
                    tokens.push(Interval(6));
                    tokens.push(Frequency(Monthly));
                    continue;
                } else {
                    return Err(ParseError::UnsupportedPattern(input.into()));
                }
            }

            // Collect tokens for "until <Month> <day> <year>"
            if until {
                match &token {
                    Month(m) => {
                        until_month = Some(m.as_u32());
                        continue;
                    }
                    Interval(n) if until_month.is_some() && until_day.is_none() => {
                        until_day = Some(*n as u32);
                        continue;
                    }
                    Interval(n) if until_month.is_some() && until_day.is_some() => {
                        let date = NaiveDate::from_ymd_opt(
                            *n as i32,
                            until_month.unwrap(),
                            until_day.unwrap() as u32,
                        )
                        .ok_or_else(|| {
                            ParseError::UnrecognizedInput("invalid until date".into())
                        })?;
                        tokens.push(UntilDate(date));
                        until = false;
                        until_month = None;
                        until_day = None;
                        continue;
                    }
                    _ => {}
                }
            }

            // "the N <weekday>": MonthDay followed by Weekday means it was actually OrdinalPosition
            if let Weekday(_) = &token {
                if let Some(MonthDay(n)) = tokens.last().copied() {
                    *tokens.last_mut().unwrap() = OrdinalPosition(n as i32);
                }
            }

            // "N of the month" / "N <weekday> of the month": retroactively fix preceding Interval
            if of_context {
                if let Frequency(_) = &token {
                    let len = tokens.len();
                    if len >= 2 {
                        // "N <weekday> of the month" → OrdinalPosition(N)
                        if let (Interval(n), Weekday(_)) = (tokens[len - 2], tokens[len - 1]) {
                            tokens[len - 2] = OrdinalPosition(n as i32);
                        }
                    } else if let Some(Interval(n)) = tokens.last().copied() {
                        // "N of the month" → MonthDay(N)
                        *tokens.last_mut().unwrap() = MonthDay(n as u8);
                    }
                    of_context = false;
                }
            }

            if matches!(token, Month(_)) {
                month_context = true;
            }
            tokens.push(token);
        }
    }

    Ok(tokens)
}

fn is_time(s: &str) -> bool {
    let s = s
        .strip_suffix("am")
        .or_else(|| s.strip_suffix("pm"))
        .unwrap_or(s);
    let mut parts = s.splitn(2, ':');
    match (parts.next(), parts.next()) {
        (Some(h), Some(m)) => {
            matches!(h.len(), 1..=2)
                && m.len() == 2
                && h.chars().all(|c| c.is_ascii_digit())
                && m.chars().all(|c| c.is_ascii_digit())
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::DaySet::*;
    use crate::token::FreqWord::*;
    use crate::token::Month::*;
    use crate::token::Weekday::*;
    use Token::*;
    use chrono::{NaiveDate, NaiveTime};

    #[test]
    fn lowercase() {
        assert_eq!(normalize("DAILY"), "daily");
        assert_eq!(normalize("Every Week"), "every week");
    }

    #[test]
    fn strip_punctuation() {
        assert_eq!(normalize("monday, wednesday."), "monday wednesday");
        assert_eq!(normalize("every day"), "every day");
        // colons are preserved (used in time expressions like "9:30am")
        assert_eq!(normalize("9:30am"), "9:30am");
        assert_eq!(normalize(":30am"), ":30am");
        assert_eq!(normalize("9:30 AM"), "9:30 am");
    }

    #[test]
    fn collapse_whitespace() {
        assert_eq!(normalize("every   day"), "every day");
        assert_eq!(normalize("  weekly  "), "weekly");
    }

    #[test]
    fn expand_every_other() {
        assert_eq!(normalize("every other day"), "every 2 day");
        assert_eq!(normalize("every other week"), "every 2 week");
    }

    #[test]
    fn expand_alternate() {
        assert_eq!(normalize("alternate weeks"), "every 2 weeks");
    }

    #[test]
    fn expand_fortnightly() {
        assert_eq!(normalize("fortnightly"), "every 2 weeks");
        assert_eq!(normalize("fortnight"), "every 2 weeks");
    }

    #[test]
    fn expand_quarterly() {
        assert_eq!(normalize("quarterly"), "every 3 months");
    }

    #[test]
    fn expand_semiannually() {
        assert_eq!(normalize("semiannually"), "every 6 months");
        assert_eq!(normalize("semi-annually"), "every 6 months"); // hyphen stripped to space, then matched
        assert_eq!(normalize("biannually"), "every 2 years");
    }

    #[test]
    fn expand_day_abbreviations() {
        assert_eq!(normalize("mon"), "monday");
        assert_eq!(normalize("tue"), "tuesday");
        assert_eq!(normalize("tues"), "tuesday");
        assert_eq!(normalize("wed"), "wednesday");
        assert_eq!(normalize("wednsday"), "wednesday");
        assert_eq!(normalize("thu"), "thursday");
        assert_eq!(normalize("thur"), "thursday");
        assert_eq!(normalize("thurs"), "thursday");
        assert_eq!(normalize("fri"), "friday");
        assert_eq!(normalize("sat"), "saturday");
        assert_eq!(normalize("sun"), "sunday");
    }

    #[test]
    fn day_abbreviations_dont_mangle_full_names() {
        // "monday" should not become "mondayay" etc.
        assert_eq!(normalize("monday"), "monday");
        assert_eq!(normalize("tuesday"), "tuesday");
        assert_eq!(normalize("wednesday"), "wednesday");
        assert_eq!(normalize("thursday"), "thursday");
        assert_eq!(normalize("friday"), "friday");
        assert_eq!(normalize("saturday"), "saturday");
        assert_eq!(normalize("sunday"), "sunday");
    }

    #[test]
    fn expand_weekday_set_abbreviations() {
        assert_eq!(normalize("wkdays"), "weekdays");
        assert_eq!(normalize("wkday"), "weekdays");
        assert_eq!(normalize("wkdy"), "weekdays");
        assert_eq!(normalize("wknds"), "weekends");
        assert_eq!(normalize("wknd"), "weekends");
    }

    #[test]
    fn expand_month_abbreviations() {
        assert_eq!(normalize("jan"), "january");
        assert_eq!(normalize("feb"), "february");
        assert_eq!(normalize("mar"), "march");
        assert_eq!(normalize("apr"), "april");
        assert_eq!(normalize("jun"), "june");
        assert_eq!(normalize("jul"), "july");
        assert_eq!(normalize("aug"), "august");
        assert_eq!(normalize("sep"), "september");
        assert_eq!(normalize("sept"), "september");
        assert_eq!(normalize("oct"), "october");
        assert_eq!(normalize("nov"), "november");
        assert_eq!(normalize("dec"), "december");
    }

    #[test]
    fn month_abbreviations_dont_mangle_full_names() {
        assert_eq!(normalize("january"), "january");
        assert_eq!(normalize("march"), "march");
        assert_eq!(normalize("october"), "october");
    }

    #[test]
    fn expand_ordinal_words() {
        assert_eq!(normalize("first"), "1");
        assert_eq!(normalize("second"), "2");
        assert_eq!(normalize("third"), "3");
        assert_eq!(normalize("fourth"), "4");
        assert_eq!(normalize("fifth"), "5");
        assert_eq!(normalize("sixth"), "6");
        assert_eq!(normalize("last"), "-1");
    }

    #[test]
    fn strip_ordinal_suffixes() {
        assert_eq!(normalize("1st"), "1");
        assert_eq!(normalize("2nd"), "2");
        assert_eq!(normalize("3rd"), "3");
        assert_eq!(normalize("4th"), "4");
        assert_eq!(normalize("15th"), "15");
        assert_eq!(normalize("21st"), "21");
    }

    #[test]
    fn ordinal_suffixes_dont_mangle_words() {
        // "nth" shouldn't be stripped since "n" is not all-digits
        assert_eq!(normalize("nth"), "nth");
    }

    #[test]
    fn expand_once_a() {
        assert_eq!(normalize("once a week"), "every 1 week");
        assert_eq!(normalize("once a month"), "every 1 month");
        assert_eq!(normalize("once a day"), "every 1 day");
    }

    #[test]
    fn twice_a_is_not_expanded() {
        // "twice a week" means 2× per week (~every 3.5 days), not "every 2 weeks".
        // We leave it unexpanded so the pattern stage can return UnsupportedPattern.
        assert_eq!(normalize("twice a week"), "twice a week");
        assert_eq!(normalize("twice a month"), "twice a month");
    }

    #[test]
    fn combined_real_inputs() {
        assert_eq!(
            normalize("Every Mon, Wed, and Fri."),
            "every monday wednesday and friday"
        );
        assert_eq!(
            normalize("every other week on mon"),
            "every 2 week on monday"
        );
        assert_eq!(
            normalize("every 3rd month on the 4th"),
            "every 3 month on the 4"
        );
        assert_eq!(
            normalize("every last fri of the month"),
            "every -1 friday of the month"
        );
        assert_eq!(normalize("every 2nd of the month"), "every 2 of the month");
        assert_eq!(normalize("Fortnightly on Tues"), "every 2 weeks on tuesday");
        assert_eq!(normalize("Quarterly"), "every 3 months");
    }

    #[test]
    fn token_frequency() {
        assert_eq!(
            tokenize(&normalize("DAILY")),
            Ok(vec![Token::Frequency(Daily)])
        );
        assert_eq!(
            tokenize(&normalize("weekly")),
            Ok(vec![Token::Frequency(Weekly)])
        );
        assert_eq!(
            tokenize(&normalize("monthly")),
            Ok(vec![Token::Frequency(Monthly)])
        );
        assert_eq!(
            tokenize(&normalize("yearly")),
            Ok(vec![Token::Frequency(Yearly)])
        );
        assert_eq!(
            tokenize(&normalize("annualy")),
            Ok(vec![Token::Frequency(Yearly)])
        );
    }

    #[test]
    fn token_time() {
        assert_eq!(
            tokenize(&normalize("daily at 9:30am")),
            Ok(vec![
                Token::Frequency(Daily),
                Token::TimeOfDay(NaiveTime::from_hms_opt(9, 30, 0).unwrap())
            ])
        );
        assert_eq!(
            tokenize(&normalize("daily at 9:30pm")),
            Ok(vec![
                Token::Frequency(Daily),
                Token::TimeOfDay(NaiveTime::from_hms_opt(21, 30, 0).unwrap())
            ])
        );
        assert_eq!(
            tokenize(&normalize("daily at 16:30")),
            Ok(vec![
                Token::Frequency(Daily),
                Token::TimeOfDay(NaiveTime::from_hms_opt(16, 30, 0).unwrap())
            ])
        );
    }

    #[test]
    fn token_every_x_period() {
        assert_eq!(
            tokenize(&normalize("every other day")),
            Ok(vec![Interval(2), Frequency(Daily)])
        );

        assert_eq!(
            tokenize(&normalize("every 3 weeks")),
            Ok(vec![Interval(3), Frequency(Weekly)])
        );

        assert_eq!(
            tokenize(&normalize("quarterly")),
            Ok(vec![Interval(3), Frequency(Monthly)])
        );

        assert_eq!(
            tokenize(&normalize("semiannually")),
            Ok(vec![Interval(6), Frequency(Monthly)])
        );
    }

    #[test]
    fn token_weekdays() {
        assert_eq!(
            tokenize(&normalize("weekly on monday")),
            Ok(vec![Frequency(Weekly), Weekday(Monday)])
        );
        assert_eq!(
            tokenize(&normalize("every other tuesday")),
            Ok(vec![Interval(2), Weekday(Tuesday)])
        );
        assert_eq!(
            tokenize(&normalize("weekdays")),
            Ok(vec![WeekdaySet(Weekdays)])
        );
        assert_eq!(
            tokenize(&normalize("weekends")),
            Ok(vec![WeekdaySet(Weekend)])
        );
        assert_eq!(
            tokenize(&normalize("weekly on friday, saturday, monday")),
            Ok(vec![
                Frequency(Weekly),
                Weekday(Friday),
                Weekday(Saturday),
                Weekday(Monday)
            ])
        );
    }

    #[test]
    fn token_months() {
        assert_eq!(tokenize(&normalize("jan")), Ok(vec![Month(January)]));
        assert_eq!(tokenize(&normalize("feb")), Ok(vec![Month(February)]));
        assert_eq!(tokenize(&normalize("mar")), Ok(vec![Month(March)]));
        assert_eq!(tokenize(&normalize("apr")), Ok(vec![Month(April)]));
        assert_eq!(tokenize(&normalize("may")), Ok(vec![Month(May)]));
        assert_eq!(tokenize(&normalize("jun")), Ok(vec![Month(June)]));
        assert_eq!(tokenize(&normalize("jul")), Ok(vec![Month(July)]));
        assert_eq!(tokenize(&normalize("aug")), Ok(vec![Month(August)]));
        assert_eq!(tokenize(&normalize("sep")), Ok(vec![Month(September)]));
        assert_eq!(tokenize(&normalize("oct")), Ok(vec![Month(October)]));
        assert_eq!(tokenize(&normalize("nov")), Ok(vec![Month(November)]));
        assert_eq!(tokenize(&normalize("dec")), Ok(vec![Month(December)]));
    }

    #[test]
    fn token_month_followed_by_day() {
        assert_eq!(
            tokenize(&normalize("june 1")),
            Ok(vec![Month(June), MonthDay(1)])
        );
        assert_eq!(
            tokenize(&normalize("every year on june 1st")),
            Ok(vec![Frequency(Yearly), Month(June), MonthDay(1)])
        );
    }

    #[test]
    fn token_twice_a() {
        assert_eq!(
            Err(ParseError::UnsupportedPattern("twice a week".into())),
            tokenize(&normalize("twice a week"))
        );
        assert_eq!(
            Err(ParseError::UnsupportedPattern("twice a month".into())),
            tokenize(&normalize("twice a month"))
        );
        assert_eq!(
            tokenize(&normalize("twice a year")),
            Ok(vec![Interval(6), Frequency(Monthly)])
        );
    }

    #[test]
    fn token_combined_real_inputs() {
        assert_eq!(
            tokenize(&normalize("Every Mon, Wed, and Fri.")),
            Ok(vec![Weekday(Monday), Weekday(Wednesday), Weekday(Friday),])
        );
        assert_eq!(
            tokenize(&normalize("every other week on mon")),
            Ok(vec![Interval(2), Frequency(Weekly), Weekday(Monday)])
        );
        assert_eq!(
            tokenize(&normalize("every 3rd month on the 4th")),
            Ok(vec![Interval(3), Frequency(Monthly), MonthDay(4)])
        );
        assert_eq!(
            tokenize(&normalize("every last fri of the month")),
            Ok(vec![
                OrdinalPosition(-1),
                Weekday(Friday),
                Frequency(Monthly)
            ])
        );
        assert_eq!(
            tokenize(&normalize("every 2nd of the month")),
            Ok(vec![MonthDay(2), Frequency(Monthly)])
        );
        assert_eq!(
            tokenize(&normalize("Fortnightly on Tues")),
            Ok(vec![Interval(2), Frequency(Weekly), Weekday(Tuesday)])
        );
        assert_eq!(
            tokenize(&normalize("weekly on thursday three times")),
            Ok(vec![Frequency(Weekly), Weekday(Thursday), Count(3)])
        );
        assert_eq!(
            tokenize(&normalize("every day until March 6th, 2027")),
            Ok(vec![
                Frequency(Daily),
                UntilDate(NaiveDate::from_ymd_opt(2027, 3, 6).unwrap())
            ])
        );
        assert_eq!(
            tokenize(&normalize("every 2nd wednesday of the month")),
            Ok(vec![
                OrdinalPosition(2),
                Weekday(Wednesday),
                Frequency(Monthly)
            ])
        );
        assert_eq!(
            tokenize(&normalize("monthly on the third friday")),
            Ok(vec![
                Frequency(Monthly),
                OrdinalPosition(3),
                Weekday(Friday),
            ])
        );
    }
}
