use sqlx::PgPool;
use uuid::Uuid;

use crate::core::conversation::application::port::outbound::restaurant::restaurant_business_info_repository_port::RestaurantBusinessInfoRepositoryPort;
use crate::core::conversation::domain::restaurant::model::{
    BusinessFact, BusinessLocation, ContactChannel, EventSpace, Facility, OpeningHours,
    PaymentMethod, RestaurantRepositoryError,
};

use super::models::{
    BusinessFactRow, ContactChannelRow, EventSpaceRow, FacilityRow, LocationRow, OpeningHoursRow,
    PaymentMethodRow,
};
use super::query_helpers::{repository_error, weekday_from_database};

#[derive(Clone)]
pub struct PostgresBusinessInfoRepository {
    pool: PgPool,
}

impl PostgresBusinessInfoRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl RestaurantBusinessInfoRepositoryPort for PostgresBusinessInfoRepository {
    async fn opening_hours(
        &self,
        business_id: Uuid,
    ) -> Result<Vec<OpeningHours>, RestaurantRepositoryError> {
        sqlx::query_as::<_, OpeningHoursRow>(
            "select day_of_week, opens_at, closes_at, is_closed from restaurant_opening_hours where business_id = $1 order by day_of_week",
        )
        .bind(business_id)
        .fetch_all(&self.pool)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|row| OpeningHours {
                    day_of_week: weekday_from_database(row.day_of_week),
                    opens_at: row.opens_at,
                    closes_at: row.closes_at,
                    is_closed: row.is_closed,
                })
                .collect()
        })
        .map_err(repository_error)
    }

    async fn location(
        &self,
        business_id: Uuid,
    ) -> Result<Option<BusinessLocation>, RestaurantRepositoryError> {
        sqlx::query_as::<_, LocationRow>(
            "select address_line, nearby_description from business_locations where business_id = $1 order by label nulls last limit 1",
        )
        .bind(business_id)
        .fetch_optional(&self.pool)
        .await
        .map(|row| {
            row.map(|row| BusinessLocation {
                address_line: row.address_line,
                nearby_description: row.nearby_description,
            })
        })
        .map_err(repository_error)
    }

    async fn contact_channels(
        &self,
        business_id: Uuid,
    ) -> Result<Vec<ContactChannel>, RestaurantRepositoryError> {
        sqlx::query_as::<_, ContactChannelRow>(
            "select channel_type, value from contact_channels where business_id = $1 and active order by is_primary desc, channel_type",
        )
        .bind(business_id)
        .fetch_all(&self.pool)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|row| ContactChannel {
                    channel_type: row.channel_type,
                    value: row.value,
                })
                .collect()
        })
        .map_err(repository_error)
    }

    async fn payment_methods(
        &self,
        business_id: Uuid,
    ) -> Result<Vec<PaymentMethod>, RestaurantRepositoryError> {
        sqlx::query_as::<_, PaymentMethodRow>(
            "select method_code from business_payment_methods where business_id = $1 order by method_code",
        )
        .bind(business_id)
        .fetch_all(&self.pool)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|row| PaymentMethod {
                    method_code: row.method_code,
                })
                .collect()
        })
        .map_err(repository_error)
    }

    async fn facilities(
        &self,
        business_id: Uuid,
    ) -> Result<Vec<Facility>, RestaurantRepositoryError> {
        sqlx::query_as::<_, FacilityRow>(
            "select facility_code, label from business_facilities where business_id = $1 order by label",
        )
        .bind(business_id)
        .fetch_all(&self.pool)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|row| Facility {
                    facility_code: row.facility_code,
                    label: row.label,
                })
                .collect()
        })
        .map_err(repository_error)
    }

    async fn facts(
        &self,
        business_id: Uuid,
        locale: &str,
    ) -> Result<Vec<BusinessFact>, RestaurantRepositoryError> {
        sqlx::query_as::<_, BusinessFactRow>(
            r#"
            select bf.fact_type, coalesce(requested.title, fallback.title) as title,
                   coalesce(requested.content, fallback.content) as content,
                   bf.metadata
            from business_facts bf
            left join business_fact_translations requested
                on requested.fact_id = bf.id and requested.locale = $2
            left join business_fact_translations fallback
                on fallback.fact_id = bf.id and fallback.locale = 'en'
            where bf.business_id = $1 and bf.active
            order by bf.fact_type
            "#,
        )
        .bind(business_id)
        .bind(locale)
        .fetch_all(&self.pool)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|row| BusinessFact {
                    fact_type: row.fact_type,
                    title: row.title,
                    content: row.content,
                    metadata: row
                        .metadata
                        .as_object()
                        .map(|value| {
                            value
                                .iter()
                                .map(|(key, value)| {
                                    let value = value
                                        .as_str()
                                        .map(str::to_string)
                                        .unwrap_or_else(|| value.to_string());
                                    (key.clone(), value)
                                })
                                .collect()
                        })
                        .unwrap_or_default(),
                })
                .collect()
        })
        .map_err(repository_error)
    }

    async fn event_spaces(
        &self,
        business_id: Uuid,
    ) -> Result<Vec<EventSpace>, RestaurantRepositoryError> {
        sqlx::query_as::<_, EventSpaceRow>(
            r#"
            select res.name, res.description, cc.value as contact
            from restaurant_event_spaces res
            left join contact_channels cc on cc.id = res.contact_channel_id
            where res.business_id = $1
            order by res.name
            "#,
        )
        .bind(business_id)
        .fetch_all(&self.pool)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|row| EventSpace {
                    name: row.name,
                    description: row.description,
                    contact: row.contact,
                })
                .collect()
        })
        .map_err(repository_error)
    }
}
