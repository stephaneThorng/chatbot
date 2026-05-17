use chrono::Weekday;

use crate::core::conversation::domain::restaurant::model::RestaurantRepositoryError;

pub(crate) fn repository_error(error: sqlx::Error) -> RestaurantRepositoryError {
    RestaurantRepositoryError {
        message: error.to_string(),
    }
}

pub(crate) fn weekday_from_database(value: i16) -> Weekday {
    match value {
        1 => Weekday::Mon,
        2 => Weekday::Tue,
        3 => Weekday::Wed,
        4 => Weekday::Thu,
        5 => Weekday::Fri,
        6 => Weekday::Sat,
        7 => Weekday::Sun,
        _ => Weekday::Mon,
    }
}
