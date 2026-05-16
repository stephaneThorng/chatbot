use crate::core::restaurant::domain::model::Reservation;

pub(super) fn format_reservation_found(reservation: &Reservation) -> String {
    format!(
        "found:{}|{}|{}|{}|{}",
        reservation.reference,
        reservation.name,
        reservation.date,
        reservation.time.format("%H:%M"),
        reservation.people_count
    )
}
