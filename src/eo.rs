use crate::{error::ParseError, token::Token};

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

    // Per-word loop.
    let mut result: Vec<String> = Vec::new();
    for word in s.split_whitespace() {
        // Strip ordinal-adjective suffix `a` from numeric stems: 1a → 1, 15a → 1.
        let word = strip_ordinal_a(word);

        // Drop a bare "a" immediately following an all-digit token — residual
        // from "1-a" being split into "1" and "a" by the punctuation pass.
        if word == "a"
            && result
                .last()
                .is_some_and(|w| !w.is_empty() && w.chars().all(|c| c.is_ascii_digit()))
        {
            continue;
        }

        let w: &str = match word {
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
            "unua" | "unuan" => "1",
            "dua" | "duan" => "2",
            "tria" | "trian" => "3",
            "kvara" | "kvaran" => "4",
            "kvina" | "kvinan" => "5",
            "sesa" | "sesan" => "6",
            "sepa" | "sepan" => "7",
            "oka" | "okan" => "8",
            "naŭa" | "naŭan" | "naua" | "nauan" => "9",
            "deka" | "dekan" => "10",
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

fn strip_ordinal_a(w: &str) -> &str {
    if let Some(stem) = w.strip_suffix('a') {
        if !stem.is_empty() && stem.chars().all(|c| c.is_ascii_digit()) {
            return stem;
        }
    }
    w
}

pub fn tokenize(_input: &str) -> Result<Vec<Token>, ParseError> {
    todo!()
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
        assert_eq!(normalize("unua"), "1");
        assert_eq!(normalize("dua"), "2");
        assert_eq!(normalize("tria"), "3");
        assert_eq!(normalize("kvara"), "4");
        assert_eq!(normalize("kvina"), "5");
        assert_eq!(normalize("sesa"), "6");
        assert_eq!(normalize("sepa"), "7");
        assert_eq!(normalize("oka"), "8");
        assert_eq!(normalize("naŭa"), "9");
        assert_eq!(normalize("deka"), "10");
        assert_eq!(normalize("lasta"), "-1");
        assert_eq!(normalize("lastan"), "-1");
        // accusative ordinal-adjective forms
        assert_eq!(normalize("trian"), "3");
        assert_eq!(normalize("duan"), "2");
    }

    #[test]
    fn ordinal_suffix_a() {
        assert_eq!(normalize("1a"), "1");
        assert_eq!(normalize("15a"), "15");
        assert_eq!(normalize("1-a"), "1");
        assert_eq!(normalize("15-a"), "15");
    }

    #[test]
    fn combined_real_inputs() {
        assert_eq!(normalize("Ĉiun alian vendredon."), "ĉiu 2 vendredo");
        assert_eq!(
            normalize("Ĉiumonate je la tria vendredo"),
            "ĉiu monato je la 3 vendredo"
        );
        assert_eq!(
            normalize("Ĉiujare je la 1a de junio"),
            "ĉiu jaro je la 1 de junio"
        );
        // ĉiun + ordinal-adjective is equivalent to ĉiun alian for the tokenizer
        assert_eq!(normalize("ĉiun duan semajnon"), "ĉiun 2 semajno");
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
}
