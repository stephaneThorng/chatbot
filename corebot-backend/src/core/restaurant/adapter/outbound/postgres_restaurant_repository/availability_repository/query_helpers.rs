use chrono::Weekday;

use crate::core::restaurant::domain::model::{Reservation, RestaurantRepositoryError};

use super::models::ReservationRow;

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

pub(crate) fn reservation_from_row(row: ReservationRow) -> Reservation {
    Reservation {
        reference: row.reference,
        name: row.customer_name,
        date: row.reservation_date,
        time: row.reservation_time,
        people_count: row.people_count.max(0) as u32,
    }
}
