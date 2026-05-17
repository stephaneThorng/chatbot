use uuid::Uuid;

use crate::core::conversation::application::port::outbound::restaurant::menu_queries::{
    MenuDietaryQuery, MenuItemDetailsQuery, MenuQuery, PriceQuery,
};
use crate::core::conversation::application::port::outbound::restaurant::restaurant_menu_repository_port::RestaurantMenuRepositoryPort;
use crate::core::conversation::application::service::restaurant::ConversationRestaurantMenuService;
use crate::core::conversation::domain::restaurant::model::{BusinessFact, MenuItem};

use super::menu_response_formatter::{
    contains_text, filter_by_ingredient, find_exact_item, find_item, format_item_refs,
    format_items, price_euros,
};
use super::query_mapper::map_price_filter;

const MENU_REFERENCE_FACT_TYPE: &str = "menu_reference";

impl<M> ConversationRestaurantMenuService<M>
where
    M: RestaurantMenuRepositoryPort + Send + Sync,
{
    pub async fn find_menu(&self, business_id: Uuid, locale: &str, query: MenuQuery) -> String {
        if let Some(filter) = query.price_filter {
            let Ok(items) = self
                .menu_repository
                .menu_items_by_price(business_id, locale, &map_price_filter(filter))
                .await
            else {
                return self.menu_fallback(business_id, locale, None).await;
            };
            if items.is_empty() {
                return self.menu_fallback(business_id, locale, None).await;
            }
            return format!("price_results:{}", format_items(&items));
        }

        let Ok(items) = self.menu_repository.menu_items(business_id, locale).await else {
            return self.external_menu_reference(business_id, locale).await;
        };

        if let Some(item_name) = query.price_item {
            if let Some(item) = find_exact_item(&items, &item_name) {
                return format!(
                    "item_found:{}|{}|{}",
                    item.name,
                    price_euros(item.price_cents),
                    item.allergens.join(",")
                );
            }

            let ingredient_matches = filter_by_ingredient(&items, &item_name);
            if !ingredient_matches.is_empty() {
                return format!("ingredient_results:{}", format_item_refs(&ingredient_matches));
            }

            return self.menu_fallback(business_id, locale, Some(&items)).await;
        }

        format!("full_menu:{}", format_items(&items))
    }

    pub async fn find_menu_dietary(
        &self,
        business_id: Uuid,
        locale: &str,
        query: MenuDietaryQuery,
    ) -> String {
        let Ok(items) = self.menu_repository.menu_items(business_id, locale).await else {
            return "dietary_no_filter:".to_string();
        };

        if let Some(requirement) = query.dietary_requirement {
            let matches = items
                .iter()
                .filter(|item| contains_text(&item.dietary, &requirement))
                .map(|item| item.name.clone())
                .collect::<Vec<_>>();
            if matches.is_empty() {
                return format!(
                    "no_dietary:{}|{}",
                    requirement,
                    available_dietary_options(&items)
                );
            }
            return format!("dietary_results:{}|{}", requirement, matches.join(", "));
        }

        format!("dietary_no_filter:{}", available_dietary_options(&items))
    }

    pub async fn find_menu_item_details(
        &self,
        business_id: Uuid,
        locale: &str,
        query: MenuItemDetailsQuery,
    ) -> String {
        let Ok(items) = self.menu_repository.menu_items(business_id, locale).await else {
            return "details_no_filter:".to_string();
        };

        match (query.menu_item, query.allergen) {
            (Some(item_name), Some(allergen)) => {
                if let Some(item) = find_item(&items, &item_name) {
                    if contains_text(&item.allergens, &allergen) {
                        return format!("contains:{item_name}|{allergen}");
                    }
                    return format!("not_contains:{item_name}|{allergen}");
                }
                format!("item_unknown:{item_name}")
            }
            (Some(item_name), None) => {
                if let Some(item) = find_item(&items, &item_name) {
                    return format!(
                        "item_details:{}|{}|{}|{}|{}",
                        item.name,
                        price_euros(item.price_cents),
                        if item.ingredients.is_empty() {
                            "none".to_string()
                        } else {
                            item.ingredients.join(", ")
                        },
                        if item.dietary.is_empty() {
                            "none".to_string()
                        } else {
                            item.dietary.join(", ")
                        },
                        if item.allergens.is_empty() {
                            "none".to_string()
                        } else {
                            item.allergens.join(", ")
                        }
                    );
                }
                format!("item_unknown:{item_name}")
            }
            (None, Some(allergen)) => {
                let matches = items
                    .iter()
                    .filter(|item| contains_text(&item.allergens, &allergen))
                    .map(|item| item.name.clone())
                    .collect::<Vec<_>>();
                if matches.is_empty() {
                    return format!("no_allergen_match:{allergen}");
                }
                format!("allergen_found:{}|{}", allergen, matches.join(", "))
            }
            (None, None) => "details_no_filter:".to_string(),
        }
    }

    pub async fn find_price(&self, business_id: Uuid, locale: &str, query: PriceQuery) -> String {
        if let Some(filter) = query.price_filter.clone() {
            let Ok(items) = self
                .menu_repository
                .menu_items_by_price(business_id, locale, &map_price_filter(filter))
                .await
            else {
                return self.menu_fallback(business_id, locale, None).await;
            };

            if items.is_empty() {
                return self.menu_fallback(business_id, locale, None).await;
            }

            return format!("price_results:{}", format_items(&items));
        }

        let Ok(items) = self.menu_repository.menu_items(business_id, locale).await else {
            return self.external_menu_reference(business_id, locale).await;
        };

        if let Some(item_name) = query.item {
            if let Some(item) = find_exact_item(&items, &item_name) {
                return format!("item_price:{}|{}", item.name, price_euros(item.price_cents));
            }

            let ingredient_matches = filter_by_ingredient(&items, &item_name);
            if !ingredient_matches.is_empty() {
                return format!("price_results:{}", format_item_refs(&ingredient_matches));
            }

            return self.menu_fallback(business_id, locale, Some(&items)).await;
        }

        format!("price_general:{}", format_items(&items))
    }

    async fn menu_fallback(
        &self,
        business_id: Uuid,
        locale: &str,
        cached_items: Option<&[MenuItem]>,
    ) -> String {
        let external = self.external_menu_reference(business_id, locale).await;
        if external.starts_with("external_menu:") {
            return external;
        }

        if let Some(items) = cached_items {
            if !items.is_empty() {
                return format!("fallback_full_menu:{}", format_items(items));
            }
            return "menu_reference_missing:".to_string();
        }

        match self.menu_repository.menu_items(business_id, locale).await {
            Ok(items) if !items.is_empty() => format!("fallback_full_menu:{}", format_items(&items)),
            Ok(_) | Err(_) => "menu_reference_missing:".to_string(),
        }
    }

    async fn external_menu_reference(&self, business_id: Uuid, locale: &str) -> String {
        let Ok(facts) = self.business_info_repository.facts(business_id, locale).await else {
            return "menu_reference_missing:".to_string();
        };

        if let Some(reference) = menu_reference(&facts) {
            return format!(
                "external_menu:{}|{}|{}",
                reference.content,
                reference
                    .metadata
                    .get("website_url")
                    .cloned()
                    .unwrap_or_default(),
                reference.metadata.get("pdf_url").cloned().unwrap_or_default()
            );
        }

        "menu_reference_missing:".to_string()
    }
}

fn available_dietary_options(items: &[MenuItem]) -> String {
    let mut dietary = items
        .iter()
        .flat_map(|item| item.dietary.clone())
        .collect::<Vec<_>>();
    dietary.sort();
    dietary.dedup();
    dietary.join(", ")
}

fn menu_reference(facts: &[BusinessFact]) -> Option<&BusinessFact> {
    facts.iter().find(|fact| {
        fact.fact_type == MENU_REFERENCE_FACT_TYPE
            && (fact.metadata.contains_key("website_url") || fact.metadata.contains_key("pdf_url"))
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::sync::Arc;

    use uuid::Uuid;

    use super::*;
    use crate::core::conversation::application::port::outbound::restaurant::restaurant_business_info_repository_port::RestaurantBusinessInfoRepositoryPort;
    use crate::core::conversation::domain::restaurant::model::{
        BusinessLocation, ContactChannel, EventSpace, Facility, MenuPriceFilter, OpeningHours,
        PaymentMethod, RestaurantRepositoryError,
    };

    #[derive(Clone)]
    struct StubMenuRepository {
        items: Vec<MenuItem>,
        price_items: Vec<MenuItem>,
        price_error: bool,
        items_error: bool,
    }

    #[async_trait::async_trait]
    impl RestaurantMenuRepositoryPort for StubMenuRepository {
        async fn menu_items(
            &self,
            _: Uuid,
            _: &str,
        ) -> Result<Vec<MenuItem>, RestaurantRepositoryError> {
            if self.items_error {
                Err(RestaurantRepositoryError {
                    message: "menu error".to_string(),
                })
            } else {
                Ok(self.items.clone())
            }
        }

        async fn menu_items_by_price(
            &self,
            _: Uuid,
            _: &str,
            _: &MenuPriceFilter,
        ) -> Result<Vec<MenuItem>, RestaurantRepositoryError> {
            if self.price_error {
                Err(RestaurantRepositoryError {
                    message: "price error".to_string(),
                })
            } else {
                Ok(self.price_items.clone())
            }
        }
    }

    #[derive(Clone)]
    struct StubBusinessInfoRepository {
        facts: Vec<BusinessFact>,
    }

    #[async_trait::async_trait]
    impl RestaurantBusinessInfoRepositoryPort for StubBusinessInfoRepository {
        async fn opening_hours(&self, _: Uuid) -> Result<Vec<OpeningHours>, RestaurantRepositoryError> {
            Ok(vec![])
        }

        async fn location(&self, _: Uuid) -> Result<Option<BusinessLocation>, RestaurantRepositoryError> {
            Ok(None)
        }

        async fn contact_channels(&self, _: Uuid) -> Result<Vec<ContactChannel>, RestaurantRepositoryError> {
            Ok(vec![])
        }

        async fn payment_methods(&self, _: Uuid) -> Result<Vec<PaymentMethod>, RestaurantRepositoryError> {
            Ok(vec![])
        }

        async fn facilities(&self, _: Uuid) -> Result<Vec<Facility>, RestaurantRepositoryError> {
            Ok(vec![])
        }

        async fn facts(
            &self,
            _: Uuid,
            _: &str,
        ) -> Result<Vec<BusinessFact>, RestaurantRepositoryError> {
            Ok(self.facts.clone())
        }

        async fn event_spaces(&self, _: Uuid) -> Result<Vec<EventSpace>, RestaurantRepositoryError> {
            Ok(vec![])
        }
    }

    fn business_id() -> Uuid {
        Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap()
    }

    fn menu_item(
        name: &str,
        ingredients: &[&str],
        dietary: &[&str],
        price_cents: i32,
    ) -> MenuItem {
        MenuItem {
            name: name.to_string(),
            ingredients: ingredients.iter().map(|value| value.to_string()).collect(),
            dietary: dietary.iter().map(|value| value.to_string()).collect(),
            allergens: vec![],
            price_cents,
            currency: "EUR".to_string(),
        }
    }

    fn menu_service(
        items: Vec<MenuItem>,
        price_items: Vec<MenuItem>,
        facts: Vec<BusinessFact>,
    ) -> ConversationRestaurantMenuService<StubMenuRepository> {
        ConversationRestaurantMenuService::new(
            StubMenuRepository {
                items,
                price_items,
                price_error: false,
                items_error: false,
            },
            Arc::new(StubBusinessInfoRepository { facts }),
        )
    }

    #[tokio::test]
    async fn dietary_lookup_matches_normalized_tokens() {
        let service = menu_service(
            vec![menu_item("vegan curry", &["tofu"], &["gluten-free", "vegan"], 1100)],
            vec![],
            vec![],
        );

        let raw = service
            .find_menu_dietary(
                business_id(),
                "en",
                MenuDietaryQuery {
                    dietary_requirement: Some("Free_GlUten".to_string()),
                },
            )
            .await;

        assert_eq!(raw, "dietary_results:Free_GlUten|vegan curry");
    }

    #[tokio::test]
    async fn dietary_lookup_returns_alternatives_on_miss() {
        let service = menu_service(
            vec![menu_item("salad", &["lettuce"], &["vegan", "gluten-free"], 800)],
            vec![],
            vec![],
        );

        let raw = service
            .find_menu_dietary(
                business_id(),
                "en",
                MenuDietaryQuery {
                    dietary_requirement: Some("nut free".to_string()),
                },
            )
            .await;

        assert_eq!(raw, "no_dietary:nut free|gluten-free, vegan");
    }

    #[tokio::test]
    async fn menu_lookup_can_filter_by_ingredient() {
        let service = menu_service(
            vec![
                menu_item("beef burger", &["beef", "bun"], &[], 1400),
                menu_item("salad", &["lettuce"], &["vegan"], 800),
            ],
            vec![],
            vec![],
        );

        let raw = service
            .find_menu(
                business_id(),
                "en",
                MenuQuery {
                    price_item: Some("beef".to_string()),
                    price_filter: None,
                },
            )
            .await;

        assert_eq!(raw, "ingredient_results:beef burger (EUR 14)");
    }

    #[tokio::test]
    async fn missing_menu_lookup_falls_back_to_external_reference_first() {
        let mut metadata = BTreeMap::new();
        metadata.insert(
            "website_url".to_string(),
            "https://example.com/menu".to_string(),
        );
        metadata.insert(
            "pdf_url".to_string(),
            "https://example.com/menu.pdf".to_string(),
        );
        let service = menu_service(
            vec![menu_item("salad", &["lettuce"], &["vegan"], 800)],
            vec![],
            vec![BusinessFact {
                fact_type: MENU_REFERENCE_FACT_TYPE.to_string(),
                title: None,
                content: "You can view our full menu online.".to_string(),
                metadata,
            }],
        );

        let raw = service
            .find_menu(
                business_id(),
                "en",
                MenuQuery {
                    price_item: Some("beef".to_string()),
                    price_filter: None,
                },
            )
            .await;

        assert_eq!(
            raw,
            "external_menu:You can view our full menu online.|https://example.com/menu|https://example.com/menu.pdf"
        );
    }

    #[tokio::test]
    async fn missing_price_lookup_falls_back_to_full_menu_without_external_reference() {
        let service = menu_service(
            vec![menu_item("salad", &["lettuce"], &["vegan"], 800)],
            vec![],
            vec![],
        );

        let raw = service
            .find_price(
                business_id(),
                "en",
                PriceQuery {
                    item: Some("beef".to_string()),
                    price_filter: None,
                },
            )
            .await;

        assert_eq!(raw, "fallback_full_menu:salad (EUR 8)");
    }

    #[tokio::test]
    async fn item_details_include_ingredients() {
        let service = menu_service(
            vec![menu_item(
                "pizza",
                &["pizza dough", "cheese", "tomato"],
                &["vegetarian"],
                1200,
            )],
            vec![],
            vec![],
        );

        let raw = service
            .find_menu_item_details(
                business_id(),
                "en",
                MenuItemDetailsQuery {
                    menu_item: Some("pizza".to_string()),
                    allergen: None,
                },
            )
            .await;

        assert_eq!(
            raw,
            "item_details:pizza|12|pizza dough, cheese, tomato|vegetarian|none"
        );
    }
}
