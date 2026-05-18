//! Emit an RFC 5545 RRULE string from a (RecurrencePattern, Modifiers) pair.

use crate::error::ParseError;
use crate::pattern::{Modifiers, RecurrencePattern};
use crate::token::{DaySet, FreqWord, Weekday};
use chrono::Timelike;

pub fn rrule(
    pattern: &RecurrencePattern,
    modifiers: Option<&Modifiers>,
) -> Result<String, ParseError> {
    let mut parts: Vec<String> = Vec::new();

    match pattern {
        RecurrencePattern::Simple { freq, interval } => {
            parts.push(format!("FREQ={}", freq_str(*freq)));
            push_interval(&mut parts, *interval);
        }
        RecurrencePattern::ByWeekday { days, interval } => {
            parts.push("FREQ=WEEKLY".into());
            push_interval(&mut parts, *interval);
            parts.push(format!("BYDAY={}", join_weekdays(days)));
        }
        RecurrencePattern::WeekdaySet { set, interval } => {
            parts.push("FREQ=WEEKLY".into());
            push_interval(&mut parts, *interval);
            let codes = match set {
                DaySet::Weekdays => "MO,TU,WE,TH,FR",
                DaySet::Weekend => "SA,SU",
            };
            parts.push(format!("BYDAY={}", codes));
        }
        RecurrencePattern::MonthlyByDay { interval, days } => {
            parts.push("FREQ=MONTHLY".into());
            push_interval(&mut parts, *interval);
            parts.push(format!("BYMONTHDAY={}", join_u8(days)));
        }
        RecurrencePattern::MonthlyByPosition {
            interval,
            pos,
            weekday,
        } => {
            parts.push("FREQ=MONTHLY".into());
            push_interval(&mut parts, *interval);
            parts.push(format!("BYDAY={}{}", pos, weekday_code(*weekday)));
        }
        RecurrencePattern::YearlyByMonth { month, days } => {
            parts.push("FREQ=YEARLY".into());
            parts.push(format!("BYMONTH={}", month.as_u32()));
            parts.push(format!("BYMONTHDAY={}", join_u8(days)));
        }
        RecurrencePattern::YearlyByPosition {
            month,
            pos,
            weekday,
        } => {
            parts.push("FREQ=YEARLY".into());
            parts.push(format!("BYMONTH={}", month.as_u32()));
            parts.push(format!("BYDAY={}{}", pos, weekday_code(*weekday)));
        }
    }

    if let Some(m) = modifiers {
        if let Some(t) = m.time {
            parts.push(format!("BYHOUR={}", t.hour()));
            parts.push(format!("BYMINUTE={}", t.minute()));
        }
        if let Some(c) = m.count {
            parts.push(format!("COUNT={}", c));
        }
        if let Some(u) = m.until {
            parts.push(format!("UNTIL={}", u.format("%Y%m%d")));
        }
    }

    Ok(parts.join(";"))
}

fn push_interval(parts: &mut Vec<String>, interval: u32) {
    if interval != 1 {
        parts.push(format!("INTERVAL={}", interval));
    }
}

fn join_weekdays(days: &[Weekday]) -> String {
    days.iter()
        .map(|d| weekday_code(*d))
        .collect::<Vec<_>>()
        .join(",")
}

fn join_u8(xs: &[u8]) -> String {
    xs.iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn freq_str(f: FreqWord) -> &'static str {
    match f {
        FreqWord::Daily => "DAILY",
        FreqWord::Weekly => "WEEKLY",
        FreqWord::Monthly => "MONTHLY",
        FreqWord::Yearly => "YEARLY",
    }
}

fn weekday_code(w: Weekday) -> &'static str {
    match w {
        Weekday::Monday => "MO",
        Weekday::Tuesday => "TU",
        Weekday::Wednesday => "WE",
        Weekday::Thursday => "TH",
        Weekday::Friday => "FR",
        Weekday::Saturday => "SA",
        Weekday::Sunday => "SU",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::RecurrencePattern::*;
    use crate::token::DaySet::*;
    use crate::token::FreqWord::*;
    use crate::token::Month::*;
    use crate::token::Weekday::*;
    use chrono::{NaiveDate, NaiveTime};

    #[test]
    fn simple_daily() {
        let p = Simple {
            freq: Daily,
            interval: 1,
        };
        assert_eq!(rrule(&p, None).unwrap(), "FREQ=DAILY");
    }

    #[test]
    fn simple_weekly_interval() {
        let p = Simple {
            freq: Weekly,
            interval: 2,
        };
        assert_eq!(rrule(&p, None).unwrap(), "FREQ=WEEKLY;INTERVAL=2");
    }

    #[test]
    fn by_weekday() {
        let p = ByWeekday {
            days: vec![Monday, Wednesday, Friday],
            interval: 1,
        };
        assert_eq!(rrule(&p, None).unwrap(), "FREQ=WEEKLY;BYDAY=MO,WE,FR");
    }

    #[test]
    fn weekday_set_weekdays() {
        let p = WeekdaySet {
            set: Weekdays,
            interval: 1,
        };
        assert_eq!(rrule(&p, None).unwrap(), "FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR");
    }

    #[test]
    fn weekday_set_weekend() {
        let p = WeekdaySet {
            set: Weekend,
            interval: 1,
        };
        assert_eq!(rrule(&p, None).unwrap(), "FREQ=WEEKLY;BYDAY=SA,SU");
    }

    #[test]
    fn monthly_by_day_single() {
        let p = MonthlyByDay {
            interval: 1,
            days: vec![15],
        };
        assert_eq!(rrule(&p, None).unwrap(), "FREQ=MONTHLY;BYMONTHDAY=15");
    }

    #[test]
    fn monthly_by_day_multi_with_interval() {
        let p = MonthlyByDay {
            interval: 2,
            days: vec![1, 15],
        };
        assert_eq!(
            rrule(&p, None).unwrap(),
            "FREQ=MONTHLY;INTERVAL=2;BYMONTHDAY=1,15"
        );
    }

    #[test]
    fn monthly_by_position_positive() {
        let p = MonthlyByPosition {
            interval: 1,
            pos: 3,
            weekday: Friday,
        };
        assert_eq!(rrule(&p, None).unwrap(), "FREQ=MONTHLY;BYDAY=3FR");
    }

    #[test]
    fn monthly_by_position_negative() {
        let p = MonthlyByPosition {
            interval: 1,
            pos: -1,
            weekday: Friday,
        };
        assert_eq!(rrule(&p, None).unwrap(), "FREQ=MONTHLY;BYDAY=-1FR");
    }

    #[test]
    fn yearly_by_month_single() {
        let p = YearlyByMonth {
            month: June,
            days: vec![1],
        };
        assert_eq!(
            rrule(&p, None).unwrap(),
            "FREQ=YEARLY;BYMONTH=6;BYMONTHDAY=1"
        );
    }

    #[test]
    fn yearly_by_position_memorial_day() {
        let p = YearlyByPosition {
            month: May,
            pos: -1,
            weekday: Monday,
        };
        assert_eq!(rrule(&p, None).unwrap(), "FREQ=YEARLY;BYMONTH=5;BYDAY=-1MO");
    }

    #[test]
    fn modifier_count() {
        let p = Simple {
            freq: Weekly,
            interval: 1,
        };
        let m = Modifiers {
            count: Some(5),
            ..Modifiers::default()
        };
        assert_eq!(rrule(&p, Some(&m)).unwrap(), "FREQ=WEEKLY;COUNT=5");
    }

    #[test]
    fn modifier_until() {
        let p = Simple {
            freq: Daily,
            interval: 1,
        };
        let m = Modifiers {
            until: Some(NaiveDate::from_ymd_opt(2027, 3, 6).unwrap()),
            ..Modifiers::default()
        };
        assert_eq!(rrule(&p, Some(&m)).unwrap(), "FREQ=DAILY;UNTIL=20270306");
    }

    #[test]
    fn modifier_time() {
        let p = Simple {
            freq: Daily,
            interval: 1,
        };
        let m = Modifiers {
            time: Some(NaiveTime::from_hms_opt(9, 30, 0).unwrap()),
            ..Modifiers::default()
        };
        assert_eq!(
            rrule(&p, Some(&m)).unwrap(),
            "FREQ=DAILY;BYHOUR=9;BYMINUTE=30"
        );
    }
}
