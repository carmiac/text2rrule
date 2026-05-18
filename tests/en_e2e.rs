//! End-to-end tests for the English locale.
//!
//! Each case runs the full pipeline with the locale forced to "en".

use text2rrule::{text2rrule_with_locale, ParseError};

fn en(input: &str) -> Result<String, ParseError> {
    text2rrule_with_locale(input, ["en".to_string()].into_iter())
}

#[test]
fn simple_daily() {
    assert_eq!(en("daily").unwrap(), "FREQ=DAILY");
}

#[test]
fn simple_every_n_weeks() {
    assert_eq!(en("every 3 weeks").unwrap(), "FREQ=WEEKLY;INTERVAL=3");
}

#[test]
fn simple_fortnightly() {
    assert_eq!(en("fortnightly").unwrap(), "FREQ=WEEKLY;INTERVAL=2");
}

#[test]
fn simple_quarterly() {
    assert_eq!(en("quarterly").unwrap(), "FREQ=MONTHLY;INTERVAL=3");
}

#[test]
fn simple_annually() {
    assert_eq!(en("annually").unwrap(), "FREQ=YEARLY");
}

#[test]
fn readme_example() {
    assert_eq!(
        en("every two weeks on friday").unwrap(),
        "FREQ=WEEKLY;INTERVAL=2;BYDAY=FR"
    );
}

#[test]
fn by_weekday_multi() {
    assert_eq!(
        en("Every Mon, Wed, and Fri.").unwrap(),
        "FREQ=WEEKLY;BYDAY=MO,WE,FR"
    );
}

#[test]
fn by_weekday_every_other() {
    assert_eq!(
        en("every other tuesday").unwrap(),
        "FREQ=WEEKLY;INTERVAL=2;BYDAY=TU"
    );
}

#[test]
fn weekday_set_weekdays() {
    assert_eq!(
        en("every weekday").unwrap(),
        "FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR"
    );
}

#[test]
fn weekday_set_weekends() {
    assert_eq!(en("weekends").unwrap(), "FREQ=WEEKLY;BYDAY=SA,SU");
}

#[test]
fn monthly_by_day() {
    assert_eq!(
        en("monthly on the 15th").unwrap(),
        "FREQ=MONTHLY;BYMONTHDAY=15"
    );
}

#[test]
fn monthly_by_position_third_friday() {
    assert_eq!(
        en("monthly on the third friday").unwrap(),
        "FREQ=MONTHLY;BYDAY=3FR"
    );
}

#[test]
fn monthly_by_position_last_friday() {
    assert_eq!(
        en("every last friday of the month").unwrap(),
        "FREQ=MONTHLY;BYDAY=-1FR"
    );
}

#[test]
fn yearly_by_month() {
    assert_eq!(
        en("every year on the 1st of june").unwrap(),
        "FREQ=YEARLY;BYMONTH=6;BYMONTHDAY=1"
    );
}

#[test]
fn yearly_by_position_memorial_day() {
    assert_eq!(
        en("every year on the last monday of may").unwrap(),
        "FREQ=YEARLY;BYMONTH=5;BYDAY=-1MO"
    );
}

#[test]
fn modifier_count() {
    assert_eq!(
        en("weekly on thursday three times").unwrap(),
        "FREQ=WEEKLY;BYDAY=TH;COUNT=3"
    );
}

#[test]
fn modifier_until() {
    assert_eq!(
        en("every day until March 6th, 2027").unwrap(),
        "FREQ=DAILY;UNTIL=20270306"
    );
}

#[test]
fn modifier_time() {
    assert_eq!(
        en("daily at 9:30am").unwrap(),
        "FREQ=DAILY;BYHOUR=9;BYMINUTE=30"
    );
}

#[test]
fn unsupported_twice_a_week() {
    assert!(matches!(
        en("twice a week"),
        Err(ParseError::UnsupportedPattern(_))
    ));
}
