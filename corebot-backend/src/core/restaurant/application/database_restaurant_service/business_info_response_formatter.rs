use uuid::Uuid;

use crate::core::restaurant::application::port::inbound::restaurant_queries::RestaurantInfoResult;
use crate::core::restaurant::application::port::outbound::restaurant_business_info_repository_port::RestaurantBusinessInfoRepositoryPort;
use crate::core::restaurant::domain::model::OpeningHours;

pub(super) async fn fact_payload<B>(
    repository: &B,
    business_id: Uuid,
    locale: &str,
    fact_type: &str,
) -> RestaurantInfoResult
where
    B: RestaurantBusinessInfoRepositoryPort + Send + Sync,
{
    let Ok(facts) = repository.facts(business_id, locale).await else {
        return RestaurantInfoResult::new(format!("{fact_type}:no|"));
    };
    let Some(fact) = facts.iter().find(|fact| fact.fact_type == fact_type) else {
        return RestaurantInfoResult::new(format!("{fact_type}:no|"));
    };
    RestaurantInfoResult::new(format!("{fact_type}:yes|{}", fact.content))
}

pub(super) fn format_opening_hours(hours: &[OpeningHours]) -> String {
    if hours.is_empty() {
        return "hours_unavailable:".to_string();
    }
    let Some(first_open) = hours.iter().find(|entry| !entry.is_closed) else {
        return "Closed".to_string();
    };
    format!(
        "Mon-Sun {} - {}",
        first_open.opens_at.format("%I:%M %P"),
        first_open.closes_at.format("%I:%M %P")
    )
}

pub(super) fn facility_matches(candidate: &str, requested: &str) -> bool {
    fn normalize(value: &str) -> String {
        value
            .to_lowercase()
            .replace("seats", "seating")
            .replace("seat", "seating")
            .replace('-', " ")
    }
    let candidate = normalize(candidate);
    let requested = normalize(requested);
    candidate.contains(&requested) || requested.contains(&candidate)
}
