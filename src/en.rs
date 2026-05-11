use crate::{error::ParseError, token::Token};

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
            "semi annually" => "every 6 months",
            "biweekly" => "every 2 weeks",
            "bimonthly" => "every 2 months",
            "biannually" => "every 2 years",

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

            // not found, must be ok as is.
            _ => w,
        };
        if !w.is_empty() {
            result.push(w.into());
        }
    }

    println!("result: {:?}", result);
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

#[cfg(test)]
mod tests {
    use super::*;

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
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, ParseError> {
    todo!()
}
