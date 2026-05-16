use std::sync::Arc;

use crate::core::conversation::application::port::outbound::restaurant::business_info_queries::{
    EventQuery as ConversationEventQuery, FacilityQuery as ConversationFacilityQuery,
    LocationQuery as ConversationLocationQuery,
    PaymentMethodQuery as ConversationPaymentMethodQuery,
};
use crate::core::conversation::application::port::outbound::restaurant::restaurant_accessibility_gateway_port::RestaurantAccessibilityGatewayPort;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_contact_gateway_port::RestaurantContactGatewayPort;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_entertainment_gateway_port::RestaurantEntertainmentGatewayPort;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_event_gateway_port::RestaurantEventGatewayPort;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_facilities_gateway_port::RestaurantFacilitiesGatewayPort;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_location_gateway_port::RestaurantLocationGatewayPort;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_opening_hours_gateway_port::RestaurantOpeningHoursGatewayPort;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_payment_methods_gateway_port::RestaurantPaymentMethodsGatewayPort;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_takeaway_gateway_port::RestaurantTakeawayGatewayPort;
use crate::core::restaurant::application::port::inbound::restaurant_information_port::RestaurantInformationUseCase as RestaurantInformationInboundPort;
use crate::core::restaurant::application::port::inbound::restaurant_queries::{
    EventQuery as RestaurantEventQuery, FacilityQuery as RestaurantFacilityQuery,
    LocationQuery as RestaurantLocationQuery, PaymentMethodQuery as RestaurantPaymentMethodQuery,
};

pub struct RestaurantBusinessInfoGateway<R: RestaurantInformationInboundPort> {
    restaurant: Arc<R>,
}

impl<R: RestaurantInformationInboundPort> RestaurantBusinessInfoGateway<R> {
    pub fn new(restaurant: Arc<R>) -> Self {
        Self { restaurant }
    }
}

#[async_trait::async_trait]
impl<R: RestaurantInformationInboundPort + Send + Sync> RestaurantOpeningHoursGatewayPort
    for RestaurantBusinessInfoGateway<R>
{
    async fn get_opening_hours(&self) -> String {
        self.restaurant.get_opening_hours().await.payload
    }
}

#[async_trait::async_trait]
impl<R: RestaurantInformationInboundPort + Send + Sync> RestaurantLocationGatewayPort
    for RestaurantBusinessInfoGateway<R>
{
    async fn find_location(&self, query: ConversationLocationQuery) -> String {
        self.restaurant
            .find_location(RestaurantLocationQuery { near: query.near })
            .await
            .payload
    }
}

#[async_trait::async_trait]
impl<R: RestaurantInformationInboundPort + Send + Sync> RestaurantContactGatewayPort
    for RestaurantBusinessInfoGateway<R>
{
    async fn get_contact(&self) -> String {
        self.restaurant.get_contact().await.payload
    }
}

#[async_trait::async_trait]
impl<R: RestaurantInformationInboundPort + Send + Sync> RestaurantPaymentMethodsGatewayPort
    for RestaurantBusinessInfoGateway<R>
{
    async fn find_payment_methods(&self, query: ConversationPaymentMethodQuery) -> String {
        self.restaurant
            .find_payment_methods(RestaurantPaymentMethodQuery {
                method: query.method,
            })
            .await
            .payload
    }
}

#[async_trait::async_trait]
impl<R: RestaurantInformationInboundPort + Send + Sync> RestaurantTakeawayGatewayPort
    for RestaurantBusinessInfoGateway<R>
{
    async fn get_takeaway_info(&self) -> String {
        self.restaurant.get_takeaway_info().await.payload
    }
}

#[async_trait::async_trait]
impl<R: RestaurantInformationInboundPort + Send + Sync> RestaurantEventGatewayPort
    for RestaurantBusinessInfoGateway<R>
{
    async fn find_event_info(&self, query: ConversationEventQuery) -> String {
        self.restaurant
            .find_event_info(RestaurantEventQuery {
                location: query.location,
            })
            .await
            .payload
    }
}

#[async_trait::async_trait]
impl<R: RestaurantInformationInboundPort + Send + Sync> RestaurantFacilitiesGatewayPort
    for RestaurantBusinessInfoGateway<R>
{
    async fn find_facility_info(&self, query: ConversationFacilityQuery) -> String {
        self.restaurant
            .find_facility_info(RestaurantFacilityQuery {
                facility: query.facility,
            })
            .await
            .payload
    }
}

#[async_trait::async_trait]
impl<R: RestaurantInformationInboundPort + Send + Sync> RestaurantAccessibilityGatewayPort
    for RestaurantBusinessInfoGateway<R>
{
    async fn get_accessibility_info(&self) -> String {
        self.restaurant.get_accessibility_info().await.payload
    }
}

#[async_trait::async_trait]
impl<R: RestaurantInformationInboundPort + Send + Sync> RestaurantEntertainmentGatewayPort
    for RestaurantBusinessInfoGateway<R>
{
    async fn get_entertainment_info(&self) -> String {
        self.restaurant.get_entertainment_info().await.payload
    }
}
