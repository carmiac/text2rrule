# text2rrule

This crate provides a way to convert plain language descriptions of recurring dates and times to RFC 5545 recurrence rules.

## Architecture Overview

The core idea is a pipeline: raw string -> normalize -> tokenize -> parse intent -> build RRule

The normalize and tokenize steps are unique for each language, but after tokenization the steps are the same.

### Stage 1: Normalization

Before any real parsing, clean the input:

lowercase, strip punctuation (commas, periods), expand contractions/abbreviations: "mon" -> "monday", "wkdy" -> "weekday" normalize numbers: "third" -> 3, "every 2nd" -> "every 2"

### Stage 2: Tokenization

Scan the normalized string into a vector semantic tokens. Something like:

```rust
enum Token {
Frequency(FreqWord), // "daily", "weekly", "monthly", "yearly"
Interval(u32), // "every 3", "every other" (= 2)
Weekday(Weekday), // "monday", "sunday", etc
Until(NaiveDate), // 'until march 3"
Count(u32),
etc...
}
```

### Stage 3: Intent

Pattern match the Vec<Token> against known recurrence styles:

```rust
enum RecurrencePattern { Simple { freq: Freq, interval: u32 }, // "every 2 weeks"

    ByWeekday { freq: Freq, interval: u32, days: Vec<Weekday> },
    // "every monday and wednesday"

    WeekdaySet { set: DaySet },
    // "every weekday", "every weekend"

    MonthlyByDay { interval: u32, day: u8 },
    // "monthly on the 15th"

    MonthlyByPosition { interval: u32, pos: i8, weekday: Weekday },
    // "monthly on the third tuesday"

    YearlyByMonth { month: Month, day: u8 },
    // "every year on june 1st"

    etc...
}
```

### Stage 4: RRule Emission

Each RecurrencePattern maps mechanically to an RRule string:

```rust
fn emit_rrule(pattern: RecurrencePattern, modifiers: Modifiers) -> String
```

Where Modifiers holds optional UNTIL, COUNT, BYHOUR etc. that can be attached to any pattern.

## Error Handling Strategy

This crate tries to catch malformed input early and will return a ParseError with a reason string.

## Language Support

Multiple languages are supported and PRs are happily accepted for more. To add a new language:

1. Add the locale entries in parser.rs: The Parser enum, get_locales(), Parser::normalize(), and Parser::tokenize()
1. Create a new locale file with pub fn's normalize and tokenize. Look at en.rs for an example.
1. Add plenty of tests for how such phrases are expressed in the target language.

## References

- Example tests from a python library: https://github.com/kvh/recurrent/blob/master/src/recurrent/test.py
- Rust rrule parser - https://github.com/fmeringdal/rust-rrule
- javascript library: https://github.com/jkbrzt/rrule/tree/master
