# Contributing

Example phrases and PRs welcome! In particular, I'd love to get more languages/locales supported.

## Architecture

The pipeline has four stages:

input -> normalize -> tokenize -> patternize -> emit -> RRULE

normalize and tokenize are per-locale, while patternize and emit are generic.

1. Normalize - Normalize the input: lowercase, strip punctuation, expand contractions and shorthands.
2. Tokenize - Turn the normalized string into a `Vec<Token>`:

```rust
enum Token {
    Frequency(FreqWord),    // "daily", "weekly", "monthly", "yearly"
    Interval(u32),          // "every 3"
    Weekday(Weekday),       // "monday", "sunday"
    WeekdaySet(DaySet),     // "weekdays", "weekends"
    MonthDay(u8),           // "the 15th"
    Month(Month),
    OrdinalPosition(i32),   // "first", "last" (-1), "third"
    UntilDate(NaiveDate),
    Count(u32),
    TimeOfDay(NaiveTime),
}
```

3. Patternize - Match the token set against known patterns:

```rust
enum RecurrencePattern {
    Simple             { freq, interval },
    ByWeekday          { days, interval },
    WeekdaySet         { set, interval },
    MonthlyByDay       { interval, days },
    MonthlyByPosition  { interval, pos, weekday },
    YearlyByMonth      { month, days },
    YearlyByPosition   { month, pos, weekday },
}
```

Any `UntilDate`, `Count`, or `TimeOfDay` tokens are pulled into a separate `Modifiers` struct that attaches to any pattern.

4. Emit - Turn a `(RecurrencePattern, Modifiers)` pair into an RRULE string.

## Adding a Locale

1. Create `src/locales/<code>.rs` exposing `pub fn normalize(&str) -> String` and `pub fn tokenize(&str) -> Result<Vec<Token>, ParseError>`. Use `src/locales/en.rs` as the reference.
2. In `src/parser.rs`, add a variant to the `Parser` enum and add it into `Parser::get_parser`, `Parser::normalize`, and `Parser::tokenize`.
3. Add tests for representative phrases. See `tests/en_e2e.rs` for end-to-end coverage.

The patternize and emit stages are locale-agnostic, a new locale only needs to produce the same `Vec<Token>` that the English tokenizer does.
