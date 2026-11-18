use crate::locales::mark_positions;
use crate::{error::ParseError, token::Token};
use std::collections::HashSet;

/// Normalize a plain-Esperanto recurrence string into a canonical form for tokenization.
///
/// Accepts three writing system and folds them all to canonical form:
///   - full Unicode with diacritics (`ĉ ĝ ĥ ĵ ŝ ŭ`)
///   - x-system (`cx gx hx jx sx ux`)
///   - bare ASCII with diacritics omitted (handled per-word in the replacement table)
pub fn normalize(input: &str) -> String {
    // Basic sanitization: lowercase, punctuation, whitespace.
    let s: String = input
        .to_lowercase()
        .chars()
        .map(|c| if ",.-()".contains(c) { ' ' } else { c })
        .collect();
    let s = s.split_whitespace().collect::<Vec<_>>().join(" ");

    // X-system fold. None of the outputs contain `x`, this is safe.
    let s = s
        .replace("cx", "ĉ")
        .replace("gx", "ĝ")
        .replace("hx", "ĥ")
        .replace("jx", "ĵ")
        .replace("sx", "ŝ")
        .replace("ux", "ŭ");

    // Multi-word shorthands.
    let s = s.replace("ĉiun alian", "ĉiu 2");
    let s = s.replace("ĉiu alia", "ĉiu 2");
    // bare-ASCII equivalents of the same phrases
    let s = s.replace("ciun alian", "ĉiu 2");
    let s = s.replace("ciu alia", "ĉiu 2");

    // Per-word loop.
    let mut result: Vec<String> = Vec::new();
    for word in s.split_whitespace() {
        // Rejoin "1-a", split into "1" and "a" by the punctuation pass, into the ordinal "1a".
        if word == "a"
            && let Some(last) = result.last_mut()
            && !last.is_empty()
            && last.chars().all(|c| c.is_ascii_digit())
        {
            last.push('a');
            continue;
        }

        let w: &str = match word {
            // Bare-ASCII "every" (x-system cxiu/cxiun already folded above).
            "ciu" => "ĉiu",
            "ciun" => "ĉiun",

            // Frequency adverbs (accented + bare-ASCII; x-system already folded above).
            "ĉiutage" | "ciutage" => "ĉiu tago",
            "ĉiusemajne" | "ciusemajne" => "ĉiu semajno",
            "ĉiumonate" | "ciumonate" => "ĉiu monato",
            "ĉiujare" | "ciujare" => "ĉiu jaro",

            // Period shorthands.
            "kvaronjare" => "ĉiu 3 monato",
            "duonjare" => "ĉiu 6 monato",
            "dusemajne" => "ĉiu 2 semajno",
            "dumonate" => "ĉiu 2 monato",
            "dujare" => "ĉiu 2 jaro",

            // Frequency nouns
            "tagon" | "tagoj" | "tagojn" => "tago",
            "semajnon" | "semajnoj" | "semajnojn" => "semajno",
            "monaton" | "monatoj" | "monatojn" => "monato",
            "jaron" | "jaroj" | "jarojn" => "jaro",

            // Weekdays
            "lunde" | "lundon" | "lundoj" | "lundojn" => "lundo",
            "marde" | "mardon" | "mardoj" | "mardojn" => "mardo",
            "merkrede" | "merkredon" | "merkredoj" | "merkredojn" => "merkredo",
            "ĵaŭde" | "ĵaŭdon" | "ĵaŭdoj" | "ĵaŭdojn" | "jaudo" | "jaude" | "jaudon" | "jaudoj"
            | "jaudojn" => "ĵaŭdo",
            "vendrede" | "vendredon" | "vendredoj" | "vendredojn" => "vendredo",
            "sabate" | "sabaton" | "sabatoj" | "sabatojn" => "sabato",
            "dimanĉe" | "dimanĉon" | "dimanĉoj" | "dimanĉojn" | "dimanco" | "dimance"
            | "dimancon" | "dimancoj" | "dimancojn" => "dimanĉo",

            // Months
            "januaron" => "januaro",
            "februaron" => "februaro",
            "marton" => "marto",
            "aprilon" => "aprilo",
            "majon" => "majo",
            "junion" => "junio",
            "julion" => "julio",
            "aŭguston" | "augusto" | "auguston" => "aŭgusto",
            "septembron" => "septembro",
            "oktobron" => "oktobro",
            "novembron" => "novembro",
            "decembron" => "decembro",

            // Day-sets.
            "labortage" | "labortagon" | "labortagoj" | "labortagojn" => "labortagoj",
            "semajnfine" | "semajnfinon" | "semajnfinoj" | "semajnfinojn" => "semajnfino",

            // Cardinal numbers.
            "unu" => "1",
            "du" => "2",
            "tri" => "3",
            "kvar" => "4",
            "kvin" => "5",
            "ses" => "6",
            "sep" => "7",
            "ok" => "8",
            "naŭ" | "nau" => "9",
            "dek" => "10",

            // Ordinal words (nominative + accusative).
            "unua" | "unuan" => "1a",
            "dua" | "duan" => "2a",
            "tria" | "trian" => "3a",
            "kvara" | "kvaran" => "4a",
            "kvina" | "kvinan" => "5a",
            "sesa" | "sesan" => "6a",
            "sepa" | "sepan" => "7a",
            "oka" | "okan" => "8a",
            "naŭa" | "naŭan" | "naua" | "nauan" => "9a",
            "deka" | "dekan" => "10a",
            "lasta" | "lastan" => "-1",

            // Count word.
            "fojo" | "fojoj" | "fojojn" => "foje",

            // Until
            "gis" => "ĝis",

            // Pass through unchanged for the tokenizer
            _ => word,
        };

        if !w.is_empty() {
            result.push(w.into());
        }
    }

    result.join(" ")
}

/// Parse a cardinal (2, -1) or ordinal (2a) number.
fn parse_number(s: &str) -> Option<i32> {
    strip_ordinal_a(s).parse().ok()
}

fn strip_ordinal_a(w: &str) -> &str {
    if let Some(stem) = w.strip_suffix('a')
        && !stem.is_empty()
        && stem.chars().all(|c| c.is_ascii_digit())
    {
        return stem;
    }
    w
}

/// Turn a normalized Esperanto string into a vector of tokens.
pub fn tokenize(input: &str) -> Result<Vec<Token>, ParseError> {
    use crate::token::DaySet::*;
    use crate::token::FreqWord::*;
    use crate::token::Month::*;
    use crate::token::Weekday::*;
    use Token::*;
    use chrono::{NaiveDate, NaiveTime};

    let mut tokens: Vec<Token> = Vec::new();

    let mut twice = false;
    let mut the_context = false;
    let mut monthday_streak = false;
    let mut month_context = false;
    let mut of_context = false;
    let mut of_pos: Option<usize> = None;
    let mut until = false;
    let mut until_month: Option<u32> = None;
    let mut until_day: Option<u32> = None;
    let mut ordinals: HashSet<usize> = HashSet::new(); // indices of tokens written as ordinals
    let mut after_every: Option<usize> = None; // index of an Interval right after "every"
    let mut prev_word: Option<&str> = None;

    for word in input.split_whitespace() {
        let follows_every = prev_word.is_some_and(|w| w == "ĉiu" || w == "ĉiun");
        prev_word = Some(word);
        let mut is_ordinal = false;

        let token: Option<Token> = match word {
            // Frequencies.
            "tago" => Some(Frequency(Daily)),
            "semajno" => Some(Frequency(Weekly)),
            "monato" => Some(Frequency(Monthly)),
            "jaro" => Some(Frequency(Yearly)),

            // Weekdays.
            "lundo" => Some(Weekday(Monday)),
            "mardo" => Some(Weekday(Tuesday)),
            "merkredo" => Some(Weekday(Wednesday)),
            "ĵaŭdo" => Some(Weekday(Thursday)),
            "vendredo" => Some(Weekday(Friday)),
            "sabato" => Some(Weekday(Saturday)),
            "dimanĉo" => Some(Weekday(Sunday)),
            "labortagoj" => Some(WeekdaySet(Weekdays)),
            "semajnfino" => Some(WeekdaySet(Weekend)),

            // Months.
            "januaro" => Some(Month(January)),
            "februaro" => Some(Month(February)),
            "marto" => Some(Month(March)),
            "aprilo" => Some(Month(April)),
            "majo" => Some(Month(May)),
            "junio" => Some(Month(June)),
            "julio" => Some(Month(July)),
            "aŭgusto" => Some(Month(August)),
            "septembro" => Some(Month(September)),
            "oktobro" => Some(Month(October)),
            "novembro" => Some(Month(November)),
            "decembro" => Some(Month(December)),

            // Filler words.
            "ĉiu" | "ĉiun" | "je" | "en" | "po" | "kaj" => continue,

            "la" => {
                the_context = true;
                continue;
            }

            "de" => {
                of_context = true;
                of_pos = Some(tokens.len());
                continue;
            }

            "ĝis" => {
                until = true;
                continue;
            }

            "dufoje" => {
                twice = true;
                continue;
            }

            // "N foje" → Count(N): rewrite the last Interval.
            "foje" => {
                if let Some(Interval(n)) = tokens.last().copied() {
                    *tokens.last_mut().unwrap() = Count(n);
                }
                continue;
            }

            // Numbers.
            s if parse_number(s).is_some() => {
                let n = parse_number(s).unwrap();
                is_ordinal = s.parse::<i32>().is_err();
                if n < 0 {
                    Some(OrdinalPosition(n))
                } else if the_context || monthday_streak {
                    the_context = false;
                    monthday_streak = true;
                    Some(MonthDay(n as u8))
                } else if month_context && !until {
                    month_context = false;
                    Some(MonthDay(n as u8))
                } else {
                    Some(Interval(n as u32))
                }
            }

            // Times.
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
            // "dufoje jaro" special case: only twice-yearly is supported.
            if twice {
                if token == Frequency(Yearly) {
                    tokens.push(Interval(6));
                    tokens.push(Frequency(Monthly));
                    continue;
                } else {
                    return Err(ParseError::UnsupportedPattern(input.into()));
                }
            }

            // Collect tokens for "ĝis <Month> <day> <year>".
            if until {
                match &token {
                    Month(m) => {
                        until_month = Some(m.as_u32());
                        continue;
                    }
                    Interval(n) if until_month.is_some() && until_day.is_none() => {
                        until_day = Some(*n);
                        continue;
                    }
                    Interval(n) if until_month.is_some() && until_day.is_some() => {
                        let date = NaiveDate::from_ymd_opt(
                            *n as i32,
                            until_month.unwrap(),
                            until_day.unwrap(),
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

            if let Weekday(_) = &token {
                mark_positions(&mut tokens, &ordinals, after_every);
            }

            // "N <weekday> de la monato" / "N <weekday> de majo": the preceding Interval is a position.
            if of_context && matches!(token, Frequency(_) | Month(_)) {
                let anchor = of_pos.unwrap_or(tokens.len());
                if anchor >= 2 {
                    if let (Interval(n), Weekday(_)) = (tokens[anchor - 2], tokens[anchor - 1]) {
                        tokens[anchor - 2] = OrdinalPosition(n as i32);
                    }
                }
            }
            if of_context && let Frequency(_) = &token {
                let anchor = of_pos.unwrap_or(tokens.len());
                // Walk back through trailing Intervals before "de" and rewrite as MonthDays.
                let mut i = anchor;
                while i > 0 {
                    if let Interval(n) = tokens[i - 1] {
                        tokens[i - 1] = MonthDay(n as u8);
                        i -= 1;
                    } else {
                        break;
                    }
                }
                of_context = false;
                of_pos = None;
            }

            if matches!(token, Month(_)) {
                month_context = true;
            }
            if !matches!(token, MonthDay(_)) {
                monthday_streak = false;
            }
            if is_ordinal {
                ordinals.insert(tokens.len());
            }
            if follows_every && matches!(token, Interval(_)) {
                after_every = Some(tokens.len());
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

    #[test]
    fn lowercase() {
        assert_eq!(normalize("ĈIUTAGE"), "ĉiu tago");
        assert_eq!(normalize("Ĉiu Semajno"), "ĉiu semajno");
    }

    #[test]
    fn strip_punctuation() {
        assert_eq!(normalize("lundo, merkredo."), "lundo merkredo");
        assert_eq!(normalize("ĉiu tago"), "ĉiu tago");
        // colons preserved for times
        assert_eq!(normalize("9:30"), "9:30");
    }

    #[test]
    fn collapse_whitespace() {
        assert_eq!(normalize("ĉiu   tago"), "ĉiu tago");
        assert_eq!(normalize("  ĉiusemajne  "), "ĉiu semajno");
    }

    #[test]
    fn x_system_fold() {
        assert_eq!(normalize("cxiutage"), "ĉiu tago");
        assert_eq!(normalize("jxauxdo"), "ĵaŭdo");
        assert_eq!(normalize("gxis"), "ĝis");
        assert_eq!(normalize("Cxiutage"), "ĉiu tago");
        assert_eq!(normalize("dimancxo"), "dimanĉo");
        assert_eq!(normalize("auxgusto"), "aŭgusto");
        assert_eq!(normalize("naux"), "9");
    }

    #[test]
    fn bare_ascii_aliases() {
        assert_eq!(normalize("jaudo"), "ĵaŭdo");
        assert_eq!(normalize("dimanco"), "dimanĉo");
        assert_eq!(normalize("augusto"), "aŭgusto");
        assert_eq!(normalize("nau"), "9");
        assert_eq!(normalize("gis"), "ĝis");
        assert_eq!(normalize("ciutage"), "ĉiu tago");
    }

    #[test]
    fn expand_ciun_alian() {
        assert_eq!(normalize("ĉiun alian semajnon"), "ĉiu 2 semajno");
        assert_eq!(normalize("ĉiu alia tago"), "ĉiu 2 tago");
    }

    #[test]
    fn expand_period_shorthands() {
        assert_eq!(normalize("kvaronjare"), "ĉiu 3 monato");
        assert_eq!(normalize("duonjare"), "ĉiu 6 monato");
        assert_eq!(normalize("dusemajne"), "ĉiu 2 semajno");
        assert_eq!(normalize("dumonate"), "ĉiu 2 monato");
        assert_eq!(normalize("dujare"), "ĉiu 2 jaro");
    }

    #[test]
    fn expand_frequency_adverbs() {
        assert_eq!(normalize("ĉiutage"), "ĉiu tago");
        assert_eq!(normalize("ĉiusemajne"), "ĉiu semajno");
        assert_eq!(normalize("ĉiumonate"), "ĉiu monato");
        assert_eq!(normalize("ĉiujare"), "ĉiu jaro");
    }

    #[test]
    fn day_inflections() {
        for variant in &["lundo", "lunde", "lundon", "lundoj", "lundojn"] {
            assert_eq!(normalize(variant), "lundo", "variant {variant}");
        }
        for variant in &["mardo", "marde", "mardon", "mardoj", "mardojn"] {
            assert_eq!(normalize(variant), "mardo");
        }
        for variant in &[
            "merkredo",
            "merkrede",
            "merkredon",
            "merkredoj",
            "merkredojn",
        ] {
            assert_eq!(normalize(variant), "merkredo");
        }
        for variant in &["ĵaŭdo", "ĵaŭde", "ĵaŭdon", "ĵaŭdoj", "ĵaŭdojn"] {
            assert_eq!(normalize(variant), "ĵaŭdo");
        }
        for variant in &[
            "vendredo",
            "vendrede",
            "vendredon",
            "vendredoj",
            "vendredojn",
        ] {
            assert_eq!(normalize(variant), "vendredo");
        }
        for variant in &["sabato", "sabate", "sabaton", "sabatoj", "sabatojn"] {
            assert_eq!(normalize(variant), "sabato");
        }
        for variant in &["dimanĉo", "dimanĉe", "dimanĉon", "dimanĉoj", "dimanĉojn"] {
            assert_eq!(normalize(variant), "dimanĉo");
        }
    }

    #[test]
    fn month_inflections() {
        assert_eq!(normalize("januaron"), "januaro");
        assert_eq!(normalize("februaron"), "februaro");
        assert_eq!(normalize("marton"), "marto");
        assert_eq!(normalize("aŭguston"), "aŭgusto");
        assert_eq!(normalize("aŭgusto"), "aŭgusto");
        assert_eq!(normalize("decembron"), "decembro");
    }

    #[test]
    fn day_sets() {
        for variant in &["labortagoj", "labortage", "labortagon", "labortagojn"] {
            assert_eq!(normalize(variant), "labortagoj");
        }
        for variant in &["semajnfino", "semajnfine", "semajnfinon", "semajnfinojn"] {
            assert_eq!(normalize(variant), "semajnfino");
        }
    }

    #[test]
    fn cardinal_words() {
        assert_eq!(normalize("unu"), "1");
        assert_eq!(normalize("du"), "2");
        assert_eq!(normalize("tri"), "3");
        assert_eq!(normalize("kvar"), "4");
        assert_eq!(normalize("kvin"), "5");
        assert_eq!(normalize("ses"), "6");
        assert_eq!(normalize("sep"), "7");
        assert_eq!(normalize("ok"), "8");
        assert_eq!(normalize("naŭ"), "9");
        assert_eq!(normalize("nau"), "9");
        assert_eq!(normalize("dek"), "10");
    }

    #[test]
    fn ordinal_words() {
        assert_eq!(normalize("unua"), "1a");
        assert_eq!(normalize("dua"), "2a");
        assert_eq!(normalize("tria"), "3a");
        assert_eq!(normalize("kvara"), "4a");
        assert_eq!(normalize("kvina"), "5a");
        assert_eq!(normalize("sesa"), "6a");
        assert_eq!(normalize("sepa"), "7a");
        assert_eq!(normalize("oka"), "8a");
        assert_eq!(normalize("naŭa"), "9a");
        assert_eq!(normalize("deka"), "10a");
        assert_eq!(normalize("lasta"), "-1");
        assert_eq!(normalize("lastan"), "-1");
        // accusative ordinal-adjective forms
        assert_eq!(normalize("trian"), "3a");
        assert_eq!(normalize("duan"), "2a");
    }

    #[test]
    fn ordinal_suffix_a() {
        assert_eq!(normalize("1a"), "1a");
        assert_eq!(normalize("15a"), "15a");
        assert_eq!(normalize("1-a"), "1a");
        assert_eq!(normalize("15-a"), "15a");
    }

    #[test]
    fn combined_real_inputs() {
        assert_eq!(normalize("Ĉiun alian vendredon."), "ĉiu 2 vendredo");
        assert_eq!(
            normalize("Ĉiumonate je la tria vendredo"),
            "ĉiu monato je la 3a vendredo"
        );
        assert_eq!(
            normalize("Ĉiujare je la 1a de junio"),
            "ĉiu jaro je la 1a de junio"
        );
        // ĉiun + ordinal-adjective is equivalent to ĉiun alian for the tokenizer
        assert_eq!(normalize("ĉiun duan semajnon"), "ĉiun 2a semajno");
        assert_eq!(normalize("cxiun alian vendredon"), "ĉiu 2 vendredo");
        assert_eq!(
            normalize("ĉiun lastan vendredon de la monato"),
            "ĉiun -1 vendredo de la monato"
        );
        assert_eq!(
            normalize("ĉiutage ĝis marto 6 2027"),
            "ĉiu tago ĝis marto 6 2027"
        );
    }

    use crate::token::DaySet::*;
    use crate::token::FreqWord::*;
    use crate::token::Month::*;
    use crate::token::Weekday::*;
    use Token::*;
    use chrono::{NaiveDate, NaiveTime};

    #[test]
    fn token_frequency() {
        assert_eq!(tokenize(&normalize("ĉiutage")), Ok(vec![Frequency(Daily)]));
        assert_eq!(
            tokenize(&normalize("ĉiusemajne")),
            Ok(vec![Frequency(Weekly)])
        );
        assert_eq!(
            tokenize(&normalize("ĉiumonate")),
            Ok(vec![Frequency(Monthly)])
        );
        assert_eq!(tokenize(&normalize("ĉiujare")), Ok(vec![Frequency(Yearly)]));
    }

    #[test]
    fn token_time() {
        assert_eq!(
            tokenize(&normalize("ĉiutage je la 9:30")),
            Ok(vec![
                Frequency(Daily),
                TimeOfDay(NaiveTime::from_hms_opt(9, 30, 0).unwrap())
            ])
        );
        assert_eq!(
            tokenize(&normalize("ĉiutage je 16:30")),
            Ok(vec![
                Frequency(Daily),
                TimeOfDay(NaiveTime::from_hms_opt(16, 30, 0).unwrap())
            ])
        );
    }

    #[test]
    fn token_every_x_period() {
        assert_eq!(
            tokenize(&normalize("ĉiun alian tagon")),
            Ok(vec![Interval(2), Frequency(Daily)])
        );
        assert_eq!(
            tokenize(&normalize("ĉiun trian semajnon")),
            Ok(vec![Interval(3), Frequency(Weekly)])
        );
        assert_eq!(
            tokenize(&normalize("kvaronjare")),
            Ok(vec![Interval(3), Frequency(Monthly)])
        );
        assert_eq!(
            tokenize(&normalize("duonjare")),
            Ok(vec![Interval(6), Frequency(Monthly)])
        );
    }

    #[test]
    fn token_weekdays() {
        assert_eq!(
            tokenize(&normalize("ĉiusemajne lunde")),
            Ok(vec![Frequency(Weekly), Weekday(Monday)])
        );
        assert_eq!(
            tokenize(&normalize("ĉiun alian mardon")),
            Ok(vec![Interval(2), Weekday(Tuesday)])
        );
        assert_eq!(
            tokenize(&normalize("labortage")),
            Ok(vec![WeekdaySet(Weekdays)])
        );
        assert_eq!(
            tokenize(&normalize("semajnfine")),
            Ok(vec![WeekdaySet(Weekend)])
        );
        assert_eq!(
            tokenize(&normalize("ĉiusemajne vendrede, sabate, lunde")),
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
        assert_eq!(
            tokenize(&normalize("januaro")),
            Ok(vec![Token::Month(January)])
        );
        assert_eq!(
            tokenize(&normalize("aŭgusto")),
            Ok(vec![Token::Month(August)])
        );
        assert_eq!(
            tokenize(&normalize("auxgusto")),
            Ok(vec![Token::Month(August)])
        );
        assert_eq!(
            tokenize(&normalize("augusto")),
            Ok(vec![Token::Month(August)])
        );
    }

    #[test]
    fn token_month_followed_by_day() {
        assert_eq!(
            tokenize(&normalize("junio 1")),
            Ok(vec![Token::Month(June), MonthDay(1)])
        );
        assert_eq!(
            tokenize(&normalize("ĉiujare je la 1a de junio")),
            Ok(vec![Frequency(Yearly), MonthDay(1), Token::Month(June)])
        );
    }

    #[test]
    fn token_twice_a() {
        assert_eq!(
            tokenize(&normalize("dufoje en jaro")),
            Ok(vec![Interval(6), Frequency(Monthly)])
        );
        assert!(matches!(
            tokenize(&normalize("dufoje en semajno")),
            Err(ParseError::UnsupportedPattern(_))
        ));
    }

    #[test]
    fn token_count() {
        assert_eq!(
            tokenize(&normalize("ĉiusemajne ĵaŭde tri foje")),
            Ok(vec![Frequency(Weekly), Weekday(Thursday), Count(3)])
        );
    }

    #[test]
    fn token_until() {
        assert_eq!(
            tokenize(&normalize("ĉiutage ĝis marto 6 2027")),
            Ok(vec![
                Frequency(Daily),
                UntilDate(NaiveDate::from_ymd_opt(2027, 3, 6).unwrap())
            ])
        );
    }

    #[test]
    fn token_ordinal_position() {
        assert_eq!(
            tokenize(&normalize("ĉiumonate je la tria vendredo")),
            Ok(vec![
                Frequency(Monthly),
                OrdinalPosition(3),
                Weekday(Friday),
            ])
        );
        assert_eq!(
            tokenize(&normalize("ĉiun lastan vendredon de la monato")),
            Ok(vec![
                OrdinalPosition(-1),
                Weekday(Friday),
                Frequency(Monthly)
            ])
        );
        assert_eq!(
            tokenize(&normalize("ĉiun duan vendredon de la monato")),
            Ok(vec![
                OrdinalPosition(2),
                Weekday(Friday),
                Frequency(Monthly)
            ])
        );
    }
}
