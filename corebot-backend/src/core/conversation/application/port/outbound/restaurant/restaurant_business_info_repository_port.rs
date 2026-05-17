use std::sync::Arc;
use uuid::Uuid;

use crate::core::conversation::domain::restaurant::model::{
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

#[async_trait::async_trait]
impl<T> RestaurantBusinessInfoRepositoryPort for Arc<T>
where
    T: RestaurantBusinessInfoRepositoryPort + Send + Sync + ?Sized,
{
    async fn opening_hours(
        &self,
        business_id: Uuid,
    ) -> Result<Vec<OpeningHours>, RestaurantRepositoryError> {
        self.as_ref().opening_hours(business_id).await
    }

    async fn location(
        &self,
        business_id: Uuid,
    ) -> Result<Option<BusinessLocation>, RestaurantRepositoryError> {
        self.as_ref().location(business_id).await
    }

    async fn contact_channels(
        &self,
        business_id: Uuid,
    ) -> Result<Vec<ContactChannel>, RestaurantRepositoryError> {
        self.as_ref().contact_channels(business_id).await
    }

    async fn payment_methods(
        &self,
        business_id: Uuid,
    ) -> Result<Vec<PaymentMethod>, RestaurantRepositoryError> {
        self.as_ref().payment_methods(business_id).await
    }

    async fn facilities(
        &self,
        business_id: Uuid,
    ) -> Result<Vec<Facility>, RestaurantRepositoryError> {
        self.as_ref().facilities(business_id).await
    }

    async fn facts(
        &self,
        business_id: Uuid,
        locale: &str,
    ) -> Result<Vec<BusinessFact>, RestaurantRepositoryError> {
        self.as_ref().facts(business_id, locale).await
    }

    async fn event_spaces(
        &self,
        business_id: Uuid,
    ) -> Result<Vec<EventSpace>, RestaurantRepositoryError> {
        self.as_ref().event_spaces(business_id).await
    }
}
