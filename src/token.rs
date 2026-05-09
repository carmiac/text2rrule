pub enum FreqWord {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

pub enum Set {
    Weekdays, // Monday-Friday
    Weekend,  // Saturday-Sunday
}

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

pub struct NaiveDate {
    day: u8,
    month: Month,
    year: u32,
}

pub struct NaiveTime {
    hour: u8,
    minute: u8,
    second: u8,
}

pub enum Token {
    Frequency(FreqWord),  // "daily", "weekly", "monthly", "yearly"
    Interval(u32),        // "every 3", "every other" (= 2)
    Weekday(Weekday),     // "monday", "tuesday", ...
    WeekdaySet(Set),      // "weekdays", "weekends"
    MonthDay(u8),         // "the 15th", "on the 1st"
    Month(Month),         // "january", "march", ...
    OrdinalPosition(i8),  // "first", "last", "third" (for "third tuesday")
    UntilDate(NaiveDate), // "until march 1st"
    Count(u32),           // "5 times", "3 occurrences"
    TimeOfDay(NaiveTime), // optional, if you want BYHOUR support
}
