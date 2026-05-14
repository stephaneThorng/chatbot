use std::sync::Arc;

use crate::core::conversation::application::port::outbound::restaurant_information_port::RestaurantInformationPort;
use crate::core::conversation::application::port::outbound::restaurant_queries::{
    EventQuery as ConversationEventQuery, FacilityQuery as ConversationFacilityQuery,
    LocationQuery as ConversationLocationQuery, MenuDietaryQuery as ConversationMenuDietaryQuery,
    MenuItemDetailsQuery as ConversationMenuItemDetailsQuery, MenuQuery as ConversationMenuQuery,
    PaymentMethodQuery as ConversationPaymentMethodQuery, PriceFilter as ConversationPriceFilter,
    PriceQuery as ConversationPriceQuery,
    ReservationCreateQuery as ConversationReservationCreateQuery,
    ReservationLookupQuery as ConversationReservationLookupQuery,
};
use crate::core::conversation::application::port::outbound::restaurant_reservation_port::RestaurantReservationPort;
use crate::core::restaurant::application::port::inbound::restaurant_information_port::RestaurantInformationPort as RestaurantInformationInboundPort;
use crate::core::restaurant::application::port::inbound::restaurant_queries::{
    EventQuery as RestaurantEventQuery, FacilityQuery as RestaurantFacilityQuery,
    LocationQuery as RestaurantLocationQuery, MenuDietaryQuery as RestaurantMenuDietaryQuery,
    MenuItemDetailsQuery as RestaurantMenuItemDetailsQuery, MenuQuery as RestaurantMenuQuery,
    PaymentMethodQuery as RestaurantPaymentMethodQuery, PriceFilter as RestaurantPriceFilter,
    PriceQuery as RestaurantPriceQuery, ReservationCreateQuery as RestaurantReservationCreateQuery,
    ReservationLookupQuery as RestaurantReservationLookupQuery,
};
use crate::core::restaurant::application::port::inbound::restaurant_reservation_port::RestaurantReservationPort as RestaurantReservationInboundPort;

pub struct RestaurantInformationGateway<R: RestaurantInformationInboundPort> {
    restaurant: Arc<R>,
}

impl<R: RestaurantInformationInboundPort> RestaurantInformationGateway<R> {
    pub fn new(restaurant: Arc<R>) -> Self {
        Self { restaurant }
    }
}

pub struct RestaurantReservationGateway<R: RestaurantReservationInboundPort> {
    restaurant: Arc<R>,
}

impl<R: RestaurantReservationInboundPort> RestaurantReservationGateway<R> {
    pub fn new(restaurant: Arc<R>) -> Self {
        Self { restaurant }
    }
}

fn map_price_filter(filter: ConversationPriceFilter) -> RestaurantPriceFilter {
    RestaurantPriceFilter {
        comparator: filter.comparator,
        amount: filter.amount,
    }
}

impl<R: RestaurantInformationInboundPort> RestaurantInformationPort
    for RestaurantInformationGateway<R>
{
    fn get_opening_hours(&self) -> String {
        self.restaurant.get_opening_hours()
    }

    fn find_menu(&self, query: ConversationMenuQuery) -> String {
        self.restaurant.find_menu(RestaurantMenuQuery {
            price_item: query.price_item,
            price_filter: query.price_filter.map(map_price_filter),
        })
    }

    fn find_menu_dietary(&self, query: ConversationMenuDietaryQuery) -> String {
        self.restaurant
            .find_menu_dietary(RestaurantMenuDietaryQuery {
                dietary_requirement: query.dietary_requirement,
            })
    }

    fn find_menu_item_details(&self, query: ConversationMenuItemDetailsQuery) -> String {
        self.restaurant
            .find_menu_item_details(RestaurantMenuItemDetailsQuery {
                menu_item: query.menu_item,
                allergen: query.allergen,
            })
    }

    fn find_location(&self, query: ConversationLocationQuery) -> String {
        self.restaurant
            .find_location(RestaurantLocationQuery { near: query.near })
    }

    fn get_contact(&self) -> String {
        self.restaurant.get_contact()
    }

    fn find_payment_methods(&self, query: ConversationPaymentMethodQuery) -> String {
        self.restaurant
            .find_payment_methods(RestaurantPaymentMethodQuery {
                method: query.method,
            })
    }

    fn find_price(&self, query: ConversationPriceQuery) -> String {
        self.restaurant.find_price(RestaurantPriceQuery {
            item: query.item,
            price_filter: query.price_filter.map(map_price_filter),
        })
    }

    fn get_takeaway_info(&self) -> String {
        self.restaurant.get_takeaway_info()
    }

    fn find_event_info(&self, query: ConversationEventQuery) -> String {
        self.restaurant.find_event_info(RestaurantEventQuery {
            location: query.location,
        })
    }

    fn find_facility_info(&self, query: ConversationFacilityQuery) -> String {
        self.restaurant.find_facility_info(RestaurantFacilityQuery {
            facility: query.facility,
        })
    }

    fn get_accessibility_info(&self) -> String {
        self.restaurant.get_accessibility_info()
    }

    fn get_entertainment_info(&self) -> String {
        self.restaurant.get_entertainment_info()
    }
}

impl<R: RestaurantReservationInboundPort> RestaurantReservationPort
    for RestaurantReservationGateway<R>
{
    fn create_reservation(&self, query: ConversationReservationCreateQuery) -> String {
        self.restaurant
            .create_reservation(RestaurantReservationCreateQuery {
                name: query.name,
                date: query.date,
                time: query.time,
                people_count: query.people_count,
            })
    }

    fn check_reservation(&self, query: ConversationReservationLookupQuery) -> String {
        self.restaurant
            .check_reservation(RestaurantReservationLookupQuery {
                reference: query.reference,
                name: query.name,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubRestaurantPort;

    impl RestaurantInformationInboundPort for StubRestaurantPort {
        fn get_opening_hours(&self) -> String {
            "Mon-Sun 9am-10pm".to_string()
        }

        fn find_menu(&self, _: RestaurantMenuQuery) -> String {
            "full_menu:pizza".to_string()
        }

        fn find_menu_dietary(&self, _: RestaurantMenuDietaryQuery) -> String {
            "dietary_no_filter:".to_string()
        }

        fn find_menu_item_details(&self, _: RestaurantMenuItemDetailsQuery) -> String {
            "details_no_filter:".to_string()
        }

        fn find_location(&self, _: RestaurantLocationQuery) -> String {
            "address:123 Main St".to_string()
        }

        fn get_contact(&self) -> String {
            "contact:+33123456789|test@example.com".to_string()
        }

        fn find_payment_methods(&self, _: RestaurantPaymentMethodQuery) -> String {
            "all_methods:cash".to_string()
        }

        fn find_price(&self, _: RestaurantPriceQuery) -> String {
            "price_general:EUR 10".to_string()
        }

        fn get_takeaway_info(&self) -> String {
            "takeaway:yes|Yes".to_string()
        }

        fn find_event_info(&self, _: RestaurantEventQuery) -> String {
            "event_info:Yes".to_string()
        }

        fn find_facility_info(&self, _: RestaurantFacilityQuery) -> String {
            "all_facilities:wifi".to_string()
        }

        fn get_accessibility_info(&self) -> String {
            "accessibility:yes|Yes".to_string()
        }

        fn get_entertainment_info(&self) -> String {
            "entertainment:yes|Live music".to_string()
        }
    }

    impl RestaurantReservationInboundPort for StubRestaurantPort {
        fn create_reservation(&self, _: RestaurantReservationCreateQuery) -> String {
            "created:REST-NEW123".to_string()
        }

        fn check_reservation(&self, _: RestaurantReservationLookupQuery) -> String {
            "no_reference:".to_string()
        }
    }

    #[test]
    fn delegates_opening_hours_to_restaurant_port() {
        let gateway = RestaurantInformationGateway::new(Arc::new(StubRestaurantPort));
        assert_eq!(gateway.get_opening_hours(), "Mon-Sun 9am-10pm");
    }

    #[test]
    fn delegates_check_reservation_to_restaurant_port() {
        let gateway = RestaurantReservationGateway::new(Arc::new(StubRestaurantPort));
        assert_eq!(
            gateway.check_reservation(ConversationReservationLookupQuery {
                reference: None,
                name: None
            }),
            "no_reference:"
        );
    }

    #[test]
    fn delegates_create_reservation_to_restaurant_port() {
        let gateway = RestaurantReservationGateway::new(Arc::new(StubRestaurantPort));
        assert_eq!(
            gateway.create_reservation(ConversationReservationCreateQuery {
                name: "Alice".to_string(),
                date: "2026-06-01".to_string(),
                time: "7pm".to_string(),
                people_count: 4,
            }),
            "created:REST-NEW123"
        );
    }
}
