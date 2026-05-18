use chrono::{NaiveDate, NaiveTime};

#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FreqWord {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DaySet {
    Weekdays, // Monday-Friday
    Weekend,  // Saturday-Sunday
}

#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Month {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

impl Month {
    pub fn as_u32(&self) -> u32 {
        match self {
            Month::January => 1,
            Month::February => 2,
            Month::March => 3,
            Month::April => 4,
            Month::May => 5,
            Month::June => 6,
            Month::July => 7,
            Month::August => 8,
            Month::September => 9,
            Month::October => 10,
            Month::November => 11,
            Month::December => 12,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Token {
    Frequency(FreqWord),  // "daily", "weekly", "monthly", "yearly"
    Interval(u32),        // "every 3", "every other" (= 2)
    Weekday(Weekday),     // "monday", "tuesday", ...
    WeekdaySet(DaySet),   // "weekdays", "weekends"
    MonthDay(u8),         // "the 15th", "on the 1st"
    Month(Month),         // "january", "march", ...
    OrdinalPosition(i32), // "first", "last", "third" (for "third tuesday")
    UntilDate(NaiveDate), // "until march 1st"
    Count(u32),           // "5 times", "3 occurrences"
    TimeOfDay(NaiveTime), // "11:30am", "16:00"
}

/// Tags for the tokens to make pattern matching easier.
#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TokenTag {
    Frequency,
    Interval,
    Weekday,
    WeekdaySet,
    MonthDay,
    Month,
    OrdinalPosition,
    UntilDate,
    Count,
    TimeOfDay,
}

impl Token {
    pub fn tag(&self) -> TokenTag {
        match self {
            Token::Frequency(_) => TokenTag::Frequency,
            Token::Interval(_) => TokenTag::Interval,
            Token::Weekday(_) => TokenTag::Weekday,
            Token::WeekdaySet(_) => TokenTag::WeekdaySet,
            Token::MonthDay(_) => TokenTag::MonthDay,
            Token::Month(_) => TokenTag::Month,
            Token::OrdinalPosition(_) => TokenTag::OrdinalPosition,
            Token::UntilDate(_) => TokenTag::UntilDate,
            Token::Count(_) => TokenTag::Count,
            Token::TimeOfDay(_) => TokenTag::TimeOfDay,
        }
    }
}
