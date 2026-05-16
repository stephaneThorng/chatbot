use crate::core::restaurant::application::database_restaurant_service::DatabaseRestaurantService;
use crate::core::restaurant::application::port::inbound::restaurant_menu_usecase::RestaurantMenuUseCase;
use crate::core::restaurant::application::port::inbound::restaurant_queries::{
    MenuDietaryQuery, MenuItemDetailsQuery, MenuItemDetailsResult, MenuQuery, MenuSearchResult,
    PriceQuery,
};
use crate::core::restaurant::application::port::outbound::restaurant_availability_repository_port::RestaurantAvailabilityRepositoryPort;
use crate::core::restaurant::application::port::outbound::restaurant_business_info_repository_port::RestaurantBusinessInfoRepositoryPort;
use crate::core::restaurant::application::port::outbound::restaurant_menu_repository_port::RestaurantMenuRepositoryPort;
use crate::core::restaurant::application::port::outbound::restaurant_reservation_repository_port::RestaurantReservationRepositoryPort;

use super::menu_response_formatter::{contains_text, find_item, format_items, price_euros};
use super::query_mapper::map_price_filter;

#[async_trait::async_trait]
impl<B, M, R, A> RestaurantMenuUseCase for DatabaseRestaurantService<B, M, R, A>
where
    B: RestaurantBusinessInfoRepositoryPort + Send + Sync,
    M: RestaurantMenuRepositoryPort + Send + Sync,
    R: RestaurantReservationRepositoryPort + Send + Sync,
    A: RestaurantAvailabilityRepositoryPort + Send + Sync,
{
    async fn find_menu(&self, query: MenuQuery) -> MenuSearchResult {
        if let Some(filter) = query.price_filter {
            let Ok(items) = self
                .menu_repository
                .menu_items_by_price(self.business_id, &self.locale, &map_price_filter(filter))
                .await
            else {
                return MenuSearchResult::new("no_results:");
            };
            if items.is_empty() {
                return MenuSearchResult::new("no_results:");
            }
            return MenuSearchResult::new(format!("price_results:{}", format_items(&items)));
        }

        let Ok(items) = self
            .menu_repository
            .menu_items(self.business_id, &self.locale)
            .await
        else {
            return MenuSearchResult::new("full_menu:");
        };

        if let Some(item_name) = query.price_item {
            if let Some(item) = find_item(&items, &item_name) {
                return MenuSearchResult::new(format!(
                    "item_found:{}|{}|{}",
                    item.name,
                    price_euros(item.price_cents),
                    item.allergens.join(",")
                ));
            }
            return MenuSearchResult::new("item_not_found:");
        }

        MenuSearchResult::new(format!("full_menu:{}", format_items(&items)))
    }

    async fn find_menu_dietary(&self, query: MenuDietaryQuery) -> MenuSearchResult {
        let Ok(items) = self
            .menu_repository
            .menu_items(self.business_id, &self.locale)
            .await
        else {
            return MenuSearchResult::new("dietary_no_filter:");
        };

        if let Some(requirement) = query.dietary_requirement {
            let req = requirement.to_lowercase();
            let matches = items
                .iter()
                .filter(|item| {
                    item.dietary
                        .iter()
                        .any(|value| value.to_lowercase().contains(&req))
                })
                .map(|item| item.name.clone())
                .collect::<Vec<_>>();
            if matches.is_empty() {
                return MenuSearchResult::new(format!("no_dietary:{requirement}"));
            }
            return MenuSearchResult::new(format!(
                "dietary_results:{}|{}",
                requirement,
                matches.join(", ")
            ));
        }

        let mut dietary = items
            .iter()
            .flat_map(|item| item.dietary.clone())
            .collect::<Vec<_>>();
        dietary.sort();
        dietary.dedup();
        MenuSearchResult::new(format!("dietary_no_filter:{}", dietary.join(", ")))
    }

    async fn find_menu_item_details(&self, query: MenuItemDetailsQuery) -> MenuItemDetailsResult {
        let Ok(items) = self
            .menu_repository
            .menu_items(self.business_id, &self.locale)
            .await
        else {
            return MenuItemDetailsResult::new("details_no_filter:");
        };

        match (query.menu_item, query.allergen) {
            (Some(item_name), Some(allergen)) => {
                if let Some(item) = find_item(&items, &item_name) {
                    if contains_text(&item.allergens, &allergen) {
                        return MenuItemDetailsResult::new(format!(
                            "contains:{item_name}|{allergen}"
                        ));
                    }
                    return MenuItemDetailsResult::new(format!(
                        "not_contains:{item_name}|{allergen}"
                    ));
                }
                MenuItemDetailsResult::new(format!("item_unknown:{item_name}"))
            }
            (Some(item_name), None) => {
                if let Some(item) = find_item(&items, &item_name) {
                    return MenuItemDetailsResult::new(format!(
                        "item_details:{}|{}|{}|{}",
                        item.name,
                        price_euros(item.price_cents),
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
                    ));
                }
                MenuItemDetailsResult::new(format!("item_unknown:{item_name}"))
            }
            (None, Some(allergen)) => {
                let matches = items
                    .iter()
                    .filter(|item| contains_text(&item.allergens, &allergen))
                    .map(|item| item.name.clone())
                    .collect::<Vec<_>>();
                if matches.is_empty() {
                    return MenuItemDetailsResult::new(format!("no_allergen_match:{allergen}"));
                }
                MenuItemDetailsResult::new(format!(
                    "allergen_found:{}|{}",
                    allergen,
                    matches.join(", ")
                ))
            }
            (None, None) => MenuItemDetailsResult::new("details_no_filter:"),
        }
    }

    async fn find_price(&self, query: PriceQuery) -> MenuSearchResult {
        self.find_menu(MenuQuery {
            price_item: query.item,
            price_filter: query.price_filter,
        })
        .await
    }
}
