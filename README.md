# text2rrule

[![CI](https://github.com/carmiac/text2rrule/actions/workflows/ci.yml/badge.svg)](https://github.com/carmiac/text2rrule/actions/workflows/ci.yml) [![Crates.io](https://img.shields.io/crates/v/text2rrule.svg)](https://crates.io/crates/text2rrule) [![Docs.rs](https://docs.rs/text2rrule/badge.svg)](https://docs.rs/text2rrule) [![MSRV](https://img.shields.io/badge/MSRV-1.85-blue.svg)](https://releases.rs/docs/1.85.0/) [![License: MPL-2.0](https://img.shields.io/badge/License-MPL_2.0-brightgreen.svg)](https://www.mozilla.org/en-US/MPL/2.0/)

A Rust crate that converts plain-language descriptions of recurring events into [RFC 5545](https://datatracker.ietf.org/doc/html/rfc5545#section-3.3.10) RRULE strings.

```text
"every two weeks on friday"      ->  FREQ=WEEKLY;INTERVAL=2;BYDAY=FR
"monthly on the third friday"    ->  FREQ=MONTHLY;BYDAY=3FR
"every weekday at 9:30am"        ->  FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR;BYHOUR=9;BYMINUTE=30
```

Still a pre-1.0 work in progress, though the API is unlikely to change much.

## Install

```bash
cargo add text2rrule
```

or for the example CLI tool:

```bash
cargo install --example text2rrule
```

Requires Rust 1.85 or newer.

## Usage

```rust
use text2rrule::text2rrule;

let rrule = text2rrule("every two weeks on friday")?;
assert_eq!(rrule, "FREQ=WEEKLY;INTERVAL=2;BYDAY=FR");
```

By default the locale is detected from the environment, with English as the fallback. To force a locale:

```rust
use text2rrule::text2rrule_with_locale;

let rrule = text2rrule_with_locale(
    "every two weeks on friday",
    ["en".to_string()],
)?;
```

## Supported Recurrence Phrases in English

| Input | RRULE |
| --- | --- |
| `daily`, `weekly`, `monthly`, `yearly` | `FREQ=DAILY`, etc... |
| `every 3 weeks`, `every other day` | `FREQ=WEEKLY;INTERVAL=3`, `FREQ=DAILY;INTERVAL=2` |
| `fortnightly`, `quarterly`, `semiannually`, `annually` | shorthand for the obvious intervals |
| `every monday and wednesday` | `FREQ=WEEKLY;BYDAY=MO,WE` |
| `every weekday`, `weekends` | `FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR` , etc... |
| `monthly on the 15th`, `the 1st and 15th of every month` | `FREQ=MONTHLY;BYMONTHDAY=15`, `BYMONTHDAY=1,15` |
| `monthly on the third friday`, `every last friday of the month` | `FREQ=MONTHLY;BYDAY=3FR`, `BYDAY=-1FR` |
| `every year on the 1st of june` | `FREQ=YEARLY;BYMONTH=6;BYMONTHDAY=1` |
| `every year on the last monday of may` | `FREQ=YEARLY;BYMONTH=5;BYDAY=-1MO` |
| `weekly on thursday three times` | `FREQ=WEEKLY;BYDAY=TH;COUNT=3` |
| `every day until March 6th, 2027` | `FREQ=DAILY;UNTIL=20270306` |
| `daily at 9:30am` | `FREQ=DAILY;BYHOUR=9;BYMINUTE=30` |

Common abbreviations and misspellings are handled (`mon`/`tues`/`wed`, `jan`/`feb`, `wkdy`, `wknd`, `wednsday`, `anually`, etc).

See `tests/en_e2e.rs` for the full set of tested inputs.

## Example CLI

A small example binary is included for trying inputs interactively:

```bash
cargo run --example text2rrule -- "every two weeks on friday"
cargo run --example text2rrule -- -v "every monday"          # verbose tracing
cargo run --example text2rrule -- -l en-us "every monday"    # force locale
```

With no input argument, it reads one line from stdin.

## Internationalization

This crate is designed to support multiple languages. However, I am only fluent enough in English and Esperanto to implement them. If you would like to help by adding a new language, see [CONTRIBUTING.md](CONTRIBUTING.md).

## Contributing

Example phrases and PRs welcome, in particular, more languages/locales. See [CONTRIBUTING.md](CONTRIBUTING.md) for the architecture overview and a recipe for adding a new locale.

## References and Inspiration

- RFC 5545 § 3.3.10 - RRULE grammar
- [rust-rrule](https://github.com/fmeringdal/rust-rrule) - the inverse direction (parsing/expanding RRULEs)
- [recurrent](https://github.com/kvh/recurrent) - Python library used as inspiration
- [rrule.js](https://github.com/jkbrzt/rrule) - JavaScript library used as inspiration

## License

[MPL 2.0](https://www.mozilla.org/en-US/MPL/2.0/)
