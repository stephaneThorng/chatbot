use crate::core::conversation::application::port::outbound::restaurant::reservation_queries::{
    ReservationCancelFailure, ReservationCancelQuery as ConversationReservationCancelQuery,
    ReservationCreateQuery as ConversationReservationCreateQuery, ReservationFailure,
    ReservationLookupQuery as ConversationReservationLookupQuery,
};
use crate::core::conversation::application::port::outbound::restaurant::restaurant_reservation_gateway_port::RestaurantReservationGatewayPort;
use crate::core::restaurant::application::port::inbound::restaurant_queries::{
    ReservationCancelQuery as RestaurantReservationCancelQuery,
    ReservationCreateQuery as RestaurantReservationCreateQuery,
    ReservationLookupQuery as RestaurantReservationLookupQuery,
};
use crate::core::restaurant::application::port::inbound::restaurant_reservation_port::RestaurantReservationUseCase as RestaurantReservationInboundPort;
use crate::core::restaurant::domain::model::{ReservationCancelError, ReservationError};

pub struct RestaurantReservationGateway<R: RestaurantReservationInboundPort> {
    restaurant: R,
}

impl<R: RestaurantReservationInboundPort> RestaurantReservationGateway<R> {
    pub fn new(restaurant: R) -> Self {
        Self { restaurant }
    }
}

fn map_reservation_error(error: ReservationError) -> ReservationFailure {
    match error {
        ReservationError::RestaurantClosed => ReservationFailure::RestaurantClosed,
        ReservationError::NoAvailability { next_slot } => ReservationFailure::NoAvailability {
            next_slot: next_slot.map(|dt| dt.format("%Y-%m-%d %H:%M").to_string()),
        },
        ReservationError::RepositoryUnavailable => {
            ReservationFailure::NoAvailability { next_slot: None }
        }
    }
}

#[async_trait::async_trait]
impl<R: RestaurantReservationInboundPort + Send + Sync> RestaurantReservationGatewayPort
    for RestaurantReservationGateway<R>
{
    async fn create_reservation(
        &self,
        query: ConversationReservationCreateQuery,
    ) -> Result<String, ReservationFailure> {
        self.restaurant
            .create_reservation(RestaurantReservationCreateQuery {
                name: query.name,
                date: query.date,
                time: query.time,
                people_count: query.people_count,
            })
            .await
            .map(|result| format!("created:{}", result.reference))
            .map_err(map_reservation_error)
    }

    async fn cancel_reservation(
        &self,
        query: ConversationReservationCancelQuery,
    ) -> Result<String, ReservationCancelFailure> {
        self.restaurant
            .cancel_reservation(RestaurantReservationCancelQuery {
                reference: query.reference,
                name: query.name,
                date: query.date,
            })
            .await
            .map(|result| format!("cancelled:{}", result.reference))
            .map_err(|error| match error {
                ReservationCancelError::NotFound
                | ReservationCancelError::RepositoryUnavailable => {
                    ReservationCancelFailure::NotFound
                }
            })
    }

    async fn check_reservation(&self, query: ConversationReservationLookupQuery) -> String {
        self.restaurant
            .check_reservation(RestaurantReservationLookupQuery {
                reference: query.reference,
                name: query.name,
            })
            .await
            .payload
    }
}
