use crate::core::conversation::application::port::outbound::restaurant::menu_queries::{
    MenuDietaryQuery as ConversationMenuDietaryQuery,
    MenuItemDetailsQuery as ConversationMenuItemDetailsQuery, MenuQuery as ConversationMenuQuery,
    PriceFilter as ConversationPriceFilter, PriceQuery as ConversationPriceQuery,
};
use crate::core::conversation::application::port::outbound::restaurant::restaurant_menu_dietary_gateway_port::RestaurantMenuDietaryGatewayPort;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_menu_item_details_gateway_port::RestaurantMenuItemDetailsGatewayPort;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_menu_gateway_port::RestaurantMenuGatewayPort;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_price_gateway_port::RestaurantPriceGatewayPort;
use crate::core::restaurant::application::port::inbound::restaurant_menu_usecase::RestaurantMenuUseCase;
use crate::core::restaurant::application::port::inbound::restaurant_queries::{
    MenuDietaryQuery as RestaurantMenuDietaryQuery,
    MenuItemDetailsQuery as RestaurantMenuItemDetailsQuery, MenuQuery as RestaurantMenuQuery,
    PriceFilter as RestaurantPriceFilter, PriceQuery as RestaurantPriceQuery,
};

pub struct RestaurantMenuGateway<R> {
    restaurant: R,
}

impl<R> RestaurantMenuGateway<R> {
    pub fn new(restaurant: R) -> Self {
        Self { restaurant }
    }
}

fn map_price_filter(filter: ConversationPriceFilter) -> RestaurantPriceFilter {
    RestaurantPriceFilter {
        comparator: filter.comparator,
        amount: filter.amount,
    }
}

#[async_trait::async_trait]
impl<R: RestaurantMenuUseCase + Send + Sync> RestaurantMenuGatewayPort
    for RestaurantMenuGateway<R>
{
    async fn find_menu(&self, query: ConversationMenuQuery) -> String {
        self.restaurant
            .find_menu(RestaurantMenuQuery {
                price_item: query.price_item,
                price_filter: query.price_filter.map(map_price_filter),
            })
            .await
            .payload
    }
}

#[async_trait::async_trait]
impl<R: RestaurantMenuUseCase + Send + Sync> RestaurantMenuDietaryGatewayPort
    for RestaurantMenuGateway<R>
{
    async fn find_menu_dietary(&self, query: ConversationMenuDietaryQuery) -> String {
        self.restaurant
            .find_menu_dietary(RestaurantMenuDietaryQuery {
                dietary_requirement: query.dietary_requirement,
            })
            .await
            .payload
    }
}

#[async_trait::async_trait]
impl<R: RestaurantMenuUseCase + Send + Sync> RestaurantMenuItemDetailsGatewayPort
    for RestaurantMenuGateway<R>
{
    async fn find_menu_item_details(&self, query: ConversationMenuItemDetailsQuery) -> String {
        self.restaurant
            .find_menu_item_details(RestaurantMenuItemDetailsQuery {
                menu_item: query.menu_item,
                allergen: query.allergen,
            })
            .await
            .payload
    }
}

#[async_trait::async_trait]
impl<R: RestaurantMenuUseCase + Send + Sync> RestaurantPriceGatewayPort
    for RestaurantMenuGateway<R>
{
    async fn find_price(&self, query: ConversationPriceQuery) -> String {
        self.restaurant
            .find_price(RestaurantPriceQuery {
                item: query.item,
                price_filter: query.price_filter.map(map_price_filter),
            })
            .await
            .payload
    }
}
