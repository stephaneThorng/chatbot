use crate::core::conversation::application::port::outbound::restaurant::business_info_queries::PaymentMethodQuery;

#[async_trait::async_trait]
pub trait RestaurantPaymentMethodsGatewayPort: Send + Sync {
    async fn find_payment_methods(&self, query: PaymentMethodQuery) -> String;
}
