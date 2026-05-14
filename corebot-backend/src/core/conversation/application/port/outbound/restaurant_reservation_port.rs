use std::sync::Arc;

use crate::core::conversation::application::port::outbound::restaurant_queries::{
    ReservationCreateQuery, ReservationFailure, ReservationLookupQuery,
};

pub trait RestaurantReservationPort {
    fn create_reservation(
        &self,
        query: ReservationCreateQuery,
    ) -> Result<String, ReservationFailure>;
    fn check_reservation(&self, query: ReservationLookupQuery) -> String;
}

impl<T: RestaurantReservationPort + Send + Sync> RestaurantReservationPort for Arc<T> {
    fn create_reservation(
        &self,
        query: ReservationCreateQuery,
    ) -> Result<String, ReservationFailure> {
        self.as_ref().create_reservation(query)
    }

    fn check_reservation(&self, query: ReservationLookupQuery) -> String {
        self.as_ref().check_reservation(query)
    }
}
