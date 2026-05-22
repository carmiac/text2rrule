//! End-to-end tests for the Esperanto locale.
//!
//! Each case runs the full pipeline with the locale forced to "eo".

use text2rrule::{ParseError, text2rrule_with_locale};

fn eo(input: &str) -> Result<String, ParseError> {
    text2rrule_with_locale(input, ["eo".to_string()])
}

#[test]
fn simple_daily() {
    assert_eq!(eo("ĉiutage").unwrap(), "FREQ=DAILY");
}

#[test]
fn simple_every_n_weeks() {
    assert_eq!(eo("ĉiun trian semajnon").unwrap(), "FREQ=WEEKLY;INTERVAL=3");
}

#[test]
fn simple_fortnightly() {
    assert_eq!(eo("dusemajne").unwrap(), "FREQ=WEEKLY;INTERVAL=2");
}

#[test]
fn simple_quarterly() {
    assert_eq!(eo("kvaronjare").unwrap(), "FREQ=MONTHLY;INTERVAL=3");
}

#[test]
fn simple_annually() {
    assert_eq!(eo("ĉiujare").unwrap(), "FREQ=YEARLY");
}

#[test]
fn readme_example() {
    assert_eq!(
        eo("ĉiun duan semajnon vendrede").unwrap(),
        "FREQ=WEEKLY;INTERVAL=2;BYDAY=FR"
    );
}

#[test]
fn by_weekday_multi() {
    assert_eq!(
        eo("Ĉiun lundon, merkredon, kaj vendredon.").unwrap(),
        "FREQ=WEEKLY;BYDAY=MO,WE,FR"
    );
}

#[test]
fn by_weekday_every_other() {
    assert_eq!(
        eo("ĉiun alian mardon").unwrap(),
        "FREQ=WEEKLY;INTERVAL=2;BYDAY=TU"
    );
}

#[test]
fn weekday_set_weekdays() {
    assert_eq!(eo("labortage").unwrap(), "FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR");
}

#[test]
fn weekday_set_weekends() {
    assert_eq!(eo("semajnfine").unwrap(), "FREQ=WEEKLY;BYDAY=SA,SU");
}

#[test]
fn monthly_by_day() {
    assert_eq!(
        eo("ĉiumonate je la 15a").unwrap(),
        "FREQ=MONTHLY;BYMONTHDAY=15"
    );
}

#[test]
fn monthly_by_day_multi_of() {
    assert_eq!(
        eo("la 1a kaj 15a de ĉiu monato").unwrap(),
        "FREQ=MONTHLY;BYMONTHDAY=1,15"
    );
}

#[test]
fn monthly_by_day_multi_on_the() {
    assert_eq!(
        eo("ĉiumonate je la 1a, 10a kaj 15a").unwrap(),
        "FREQ=MONTHLY;BYMONTHDAY=1,10,15"
    );
}

#[test]
fn monthly_by_position_third_friday() {
    assert_eq!(
        eo("ĉiumonate je la tria vendredo").unwrap(),
        "FREQ=MONTHLY;BYDAY=3FR"
    );
}

#[test]
fn monthly_by_position_last_friday() {
    assert_eq!(
        eo("ĉiun lastan vendredon de la monato").unwrap(),
        "FREQ=MONTHLY;BYDAY=-1FR"
    );
}

#[test]
fn yearly_by_month() {
    assert_eq!(
        eo("ĉiujare je la 1a de junio").unwrap(),
        "FREQ=YEARLY;BYMONTH=6;BYMONTHDAY=1"
    );
}

#[test]
fn yearly_by_position_memorial_day() {
    assert_eq!(
        eo("ĉiujare je la lasta lundo de majo").unwrap(),
        "FREQ=YEARLY;BYMONTH=5;BYDAY=-1MO"
    );
}

#[test]
fn modifier_count() {
    assert_eq!(
        eo("ĉiusemajne ĵaŭde tri foje").unwrap(),
        "FREQ=WEEKLY;BYDAY=TH;COUNT=3"
    );
}

#[test]
fn modifier_until() {
    assert_eq!(
        eo("ĉiutage ĝis marto 6 2027").unwrap(),
        "FREQ=DAILY;UNTIL=20270306"
    );
}

#[test]
fn modifier_time() {
    assert_eq!(
        eo("ĉiutage je la 9:30").unwrap(),
        "FREQ=DAILY;BYHOUR=9;BYMINUTE=30"
    );
}

#[test]
fn unsupported_twice_a_week() {
    assert!(matches!(
        eo("dufoje en semajno"),
        Err(ParseError::UnsupportedPattern(_))
    ));
}

#[test]
fn x_system_input() {
    assert_eq!(
        eo("cxiun alian vendredon").unwrap(),
        "FREQ=WEEKLY;INTERVAL=2;BYDAY=FR"
    );
}

#[test]
fn bare_ascii_input() {
    assert_eq!(
        eo("ciun alian jaudon").unwrap(),
        "FREQ=WEEKLY;INTERVAL=2;BYDAY=TH"
    );
}
