use crate::core::restaurant::application::database_restaurant_service::DatabaseRestaurantService;
use crate::core::restaurant::application::port::inbound::restaurant_business_info_usecase::RestaurantBusinessInfoUseCase;
use crate::core::restaurant::application::port::inbound::restaurant_queries::{
    EventQuery, FacilityQuery, FacilityResult, LocationQuery, PaymentMethodQuery,
    PaymentMethodsResult, RestaurantInfoResult,
};
use crate::core::restaurant::application::port::outbound::restaurant_availability_repository_port::RestaurantAvailabilityRepositoryPort;
use crate::core::restaurant::application::port::outbound::restaurant_business_info_repository_port::RestaurantBusinessInfoRepositoryPort;
use crate::core::restaurant::application::port::outbound::restaurant_menu_repository_port::RestaurantMenuRepositoryPort;
use crate::core::restaurant::application::port::outbound::restaurant_reservation_repository_port::RestaurantReservationRepositoryPort;

use super::business_info_response_formatter::{facility_matches, fact_payload};

#[async_trait::async_trait]
impl<B, M, R, A> RestaurantBusinessInfoUseCase for DatabaseRestaurantService<B, M, R, A>
where
    B: RestaurantBusinessInfoRepositoryPort + Send + Sync,
    M: RestaurantMenuRepositoryPort + Send + Sync,
    R: RestaurantReservationRepositoryPort + Send + Sync,
    A: RestaurantAvailabilityRepositoryPort + Send + Sync,
{
    async fn find_location(&self, query: LocationQuery) -> RestaurantInfoResult {
        let Ok(location) = self
            .business_info_repository
            .location(self.business_id)
            .await
        else {
            return RestaurantInfoResult::new("address:");
        };
        let Some(location) = location else {
            return RestaurantInfoResult::new("address:");
        };
        let address = match location.nearby_description {
            Some(nearby) if !nearby.is_empty() => format!("{} - {}", location.address_line, nearby),
            _ => location.address_line,
        };
        if let Some(near) = query.near {
            let normalized = near.to_lowercase();
            if address.to_lowercase().contains(&normalized) {
                return RestaurantInfoResult::new(format!("near_confirmed:{near}|{address}"));
            }
            return RestaurantInfoResult::new(format!("near_denied:{near}|{address}"));
        }
        RestaurantInfoResult::new(format!("address:{address}"))
    }

    async fn get_contact(&self) -> RestaurantInfoResult {
        let Ok(channels) = self
            .business_info_repository
            .contact_channels(self.business_id)
            .await
        else {
            return RestaurantInfoResult::new("contact:|");
        };
        let phone = channels
            .iter()
            .find(|channel| channel.channel_type == "phone")
            .map(|channel| channel.value.as_str())
            .unwrap_or("");
        let email = channels
            .iter()
            .find(|channel| channel.channel_type == "email")
            .map(|channel| channel.value.as_str())
            .unwrap_or("");
        RestaurantInfoResult::new(format!("contact:{phone}|{email}"))
    }

    async fn find_payment_methods(&self, query: PaymentMethodQuery) -> PaymentMethodsResult {
        let Ok(methods) = self
            .business_info_repository
            .payment_methods(self.business_id)
            .await
        else {
            return PaymentMethodsResult::new("all_methods:");
        };
        let all = methods
            .iter()
            .map(|method| method.method_code.clone())
            .collect::<Vec<_>>()
            .join(", ");
        if let Some(method) = query.method {
            if methods.iter().any(|candidate| {
                candidate
                    .method_code
                    .to_lowercase()
                    .contains(&method.to_lowercase())
            }) {
                return PaymentMethodsResult::new(format!("method_accepted:{method}|{all}"));
            }
            return PaymentMethodsResult::new(format!("method_not_accepted:{method}|{all}"));
        }
        PaymentMethodsResult::new(format!("all_methods:{all}"))
    }

    async fn get_takeaway_info(&self) -> RestaurantInfoResult {
        fact_payload(
            &self.business_info_repository,
            self.business_id,
            &self.locale,
            "takeaway",
        )
        .await
    }

    async fn find_event_info(&self, query: EventQuery) -> RestaurantInfoResult {
        let Ok(spaces) = self
            .business_info_repository
            .event_spaces(self.business_id)
            .await
        else {
            return RestaurantInfoResult::new("event_info:");
        };
        if let Some(location) = query.location {
            if let Some(space) = spaces
                .iter()
                .find(|space| location.to_lowercase().contains(&space.name.to_lowercase()))
            {
                let info = space
                    .contact
                    .clone()
                    .or_else(|| space.description.clone())
                    .unwrap_or_default();
                return RestaurantInfoResult::new(format!(
                    "event_space_available:{location}|{info}"
                ));
            }
            return RestaurantInfoResult::new(format!(
                "event_space_unavailable:{}|We have {} available for events.",
                location,
                spaces
                    .iter()
                    .map(|space| space.name.clone())
                    .collect::<Vec<_>>()
                    .join(" and ")
            ));
        }
        RestaurantInfoResult::new(format!(
            "event_info:{}",
            spaces
                .iter()
                .map(|space| {
                    let description = space.description.clone().unwrap_or_default();
                    format!("{} {}", space.name, description).trim().to_string()
                })
                .collect::<Vec<_>>()
                .join(", ")
        ))
    }

    async fn find_facility_info(&self, query: FacilityQuery) -> FacilityResult {
        let Ok(facilities) = self
            .business_info_repository
            .facilities(self.business_id)
            .await
        else {
            return FacilityResult::new("all_facilities:");
        };
        if let Some(facility) = query.facility {
            if facilities
                .iter()
                .any(|candidate| facility_matches(&candidate.label, &facility))
            {
                return FacilityResult::new(format!("facility_available:{facility}"));
            }
            return FacilityResult::new(format!("facility_unavailable:{facility}"));
        }
        FacilityResult::new(format!(
            "all_facilities:{}",
            facilities
                .iter()
                .map(|facility| facility.label.clone())
                .collect::<Vec<_>>()
                .join(", ")
        ))
    }

    async fn get_accessibility_info(&self) -> RestaurantInfoResult {
        fact_payload(
            &self.business_info_repository,
            self.business_id,
            &self.locale,
            "accessibility",
        )
        .await
    }

    async fn get_entertainment_info(&self) -> RestaurantInfoResult {
        fact_payload(
            &self.business_info_repository,
            self.business_id,
            &self.locale,
            "entertainment",
        )
        .await
    }
}
