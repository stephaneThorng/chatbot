use crate::core::conversation::application::port::outbound::restaurant::business_info_queries::FacilityQuery;

#[async_trait::async_trait]
pub trait RestaurantFacilitiesGatewayPort: Send + Sync {
    async fn find_facility_info(&self, query: FacilityQuery) -> String;
}
