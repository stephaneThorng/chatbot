use chrono::{Datelike, NaiveDate, NaiveTime, Timelike, Weekday};

pub fn format_date(date: NaiveDate, lang: &str) -> String {
    rust_i18n::t!(
        "datetime.date.long",
        locale = lang,
        weekday = weekday_name(date.weekday(), lang),
        month = month_name(date.month(), lang),
        day = date.day(),
        year = date.year()
    )
    .to_string()
}

pub fn format_time(time: NaiveTime, lang: &str) -> String {
    rust_i18n::t!(
        "datetime.time.hm",
        locale = lang,
        hour = format!("{:02}", time.hour()),
        minute = format!("{:02}", time.minute())
    )
    .to_string()
}

fn weekday_name(weekday: Weekday, lang: &str) -> String {
    rust_i18n::t!(weekday_key(weekday), locale = lang).to_string()
}

fn weekday_key(weekday: Weekday) -> &'static str {
    match weekday {
        Weekday::Mon => "datetime.weekday.mon",
        Weekday::Tue => "datetime.weekday.tue",
        Weekday::Wed => "datetime.weekday.wed",
        Weekday::Thu => "datetime.weekday.thu",
        Weekday::Fri => "datetime.weekday.fri",
        Weekday::Sat => "datetime.weekday.sat",
        Weekday::Sun => "datetime.weekday.sun",
    }
}

fn month_name(month: u32, lang: &str) -> String {
    rust_i18n::t!(month_key(month), locale = lang).to_string()
}

fn month_key(month: u32) -> &'static str {
    match month {
        1 => "datetime.month.jan",
        2 => "datetime.month.feb",
        3 => "datetime.month.mar",
        4 => "datetime.month.apr",
        5 => "datetime.month.may",
        6 => "datetime.month.jun",
        7 => "datetime.month.jul",
        8 => "datetime.month.aug",
        9 => "datetime.month.sep",
        10 => "datetime.month.oct",
        11 => "datetime.month.nov",
        12 => "datetime.month.dec",
        _ => "datetime.month.unknown",
    }
}
