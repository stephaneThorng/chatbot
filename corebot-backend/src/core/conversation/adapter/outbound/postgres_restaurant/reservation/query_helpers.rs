use crate::core::conversation::domain::restaurant::model::{
    Reservation, RestaurantRepositoryError,
};

use super::models::ReservationRow;

pub(crate) fn repository_error(error: sqlx::Error) -> RestaurantRepositoryError {
    RestaurantRepositoryError {
        message: error.to_string(),
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
