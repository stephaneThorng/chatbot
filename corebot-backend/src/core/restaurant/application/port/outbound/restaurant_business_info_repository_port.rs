use uuid::Uuid;

use crate::core::restaurant::domain::model::{
    BusinessFact, BusinessLocation, ContactChannel, EventSpace, Facility, OpeningHours,
    PaymentMethod, RestaurantRepositoryError,
};

#[async_trait::async_trait]
pub trait RestaurantBusinessInfoRepositoryPort {
    async fn opening_hours(
        &self,
        business_id: Uuid,
    ) -> Result<Vec<OpeningHours>, RestaurantRepositoryError>;
    async fn location(
        &self,
        business_id: Uuid,
    ) -> Result<Option<BusinessLocation>, RestaurantRepositoryError>;
    async fn contact_channels(
        &self,
        business_id: Uuid,
    ) -> Result<Vec<ContactChannel>, RestaurantRepositoryError>;
    async fn payment_methods(
        &self,
        business_id: Uuid,
    ) -> Result<Vec<PaymentMethod>, RestaurantRepositoryError>;
    async fn facilities(
        &self,
        business_id: Uuid,
    ) -> Result<Vec<Facility>, RestaurantRepositoryError>;
    async fn facts(
        &self,
        business_id: Uuid,
        locale: &str,
    ) -> Result<Vec<BusinessFact>, RestaurantRepositoryError>;
    async fn event_spaces(
        &self,
        business_id: Uuid,
    ) -> Result<Vec<EventSpace>, RestaurantRepositoryError>;
}
