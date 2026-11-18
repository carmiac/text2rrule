# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.3] - 2026-11-18

### Added

- Multiple ordinal weekdays in monthly and yearly rules (e.g. `first and third monday` -> `FREQ=MONTHLY;BYDAY=1MO,3MO`, `first monday and third wednesday` -> `FREQ=MONTHLY;BYDAY=1MO,3WE`, `the first and third monday of may` -> `FREQ=YEARLY;BYMONTH=5;BYDAY=1MO,3MO`).

## [0.1.2] - 2026-05-22

### Added

- Multiple days of the month in monthly rules (e.g. `the 1st and 15th of every month` -> `FREQ=MONTHLY;BYMONTHDAY=1,15`).

## [0.1.1] - 2026-05-20

### Fixed

- Replace experimental `if let` guards with stable equivalents so the crate builds on stable Rust.

## [0.1.0] - 2026-05-18

### Added

- Initial release: English and Esperanto tokenizers, locale-agnostic patternize and emit stages, and an example CLI binary.

[Unreleased]: https://github.com/carmiac/text2rrule/compare/v0.1.3...HEAD
[0.1.3]: https://github.com/carmiac/text2rrule/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/carmiac/text2rrule/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/carmiac/text2rrule/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/carmiac/text2rrule/releases/tag/v0.1.0
