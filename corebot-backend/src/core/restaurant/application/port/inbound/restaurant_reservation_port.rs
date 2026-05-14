use crate::core::restaurant::application::port::inbound::restaurant_queries::{
    ReservationCreateQuery, ReservationLookupQuery,
};

pub trait RestaurantReservationPort {
    fn create_reservation(&self, query: ReservationCreateQuery) -> String;
    fn check_reservation(&self, query: ReservationLookupQuery) -> String;
}
