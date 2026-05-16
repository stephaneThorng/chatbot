use crate::core::conversation::application::intent_handler::intent_handler::{
    IntentHandler, IntentHandlerRegistry,
};
use crate::core::conversation::application::intent_handler::handler::ask_accessibility_handler::AskAccessibilityIntentHandler;
use crate::core::conversation::application::intent_handler::handler::ask_contact_handler::AskContactIntentHandler;
use crate::core::conversation::application::intent_handler::handler::ask_entertainment_handler::AskEntertainmentIntentHandler;
use crate::core::conversation::application::intent_handler::handler::ask_event_handler::AskEventIntentHandler;
use crate::core::conversation::application::intent_handler::handler::ask_facilities_handler::AskFacilitiesIntentHandler;
use crate::core::conversation::application::intent_handler::handler::ask_location_handler::AskLocationIntentHandler;
use crate::core::conversation::application::intent_handler::handler::ask_menu_dietary_handler::AskMenuDietaryIntentHandler;
use crate::core::conversation::application::intent_handler::handler::ask_menu_general_handler::AskMenuGeneralIntentHandler;
use crate::core::conversation::application::intent_handler::handler::ask_payment_methods_handler::AskPaymentMethodsIntentHandler;
use crate::core::conversation::application::intent_handler::handler::ask_price_handler::AskPriceIntentHandler;
use crate::core::conversation::application::intent_handler::handler::ask_takeaway_delivery_handler::AskTakeawayDeliveryIntentHandler;
use crate::core::conversation::application::intent_handler::handler::check_reservation_handler::CheckReservationIntentHandler;
use crate::core::conversation::application::intent_handler::handler::menu_item_details_handler::MenuItemDetailsIntentHandler;
use crate::core::conversation::application::intent_handler::handler::opening_hours_handler::OpeningHoursIntentHandler;
use crate::core::conversation::application::intent_handler::handler::reservation_cancel_handler::ReservationCancelIntentHandler;
use crate::core::conversation::application::intent_handler::handler::reservation_create_handler::ReservationCreateIntentHandler;
use crate::core::conversation::application::intent_handler::handler::static_reply_handler::StaticReplyIntentHandler;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_accessibility_gateway_port::RestaurantAccessibilityGatewayPort;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_contact_gateway_port::RestaurantContactGatewayPort;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_entertainment_gateway_port::RestaurantEntertainmentGatewayPort;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_event_gateway_port::RestaurantEventGatewayPort;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_facilities_gateway_port::RestaurantFacilitiesGatewayPort;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_location_gateway_port::RestaurantLocationGatewayPort;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_menu_dietary_gateway_port::RestaurantMenuDietaryGatewayPort;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_menu_item_details_gateway_port::RestaurantMenuItemDetailsGatewayPort;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_menu_gateway_port::RestaurantMenuGatewayPort;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_opening_hours_gateway_port::RestaurantOpeningHoursGatewayPort;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_payment_methods_gateway_port::RestaurantPaymentMethodsGatewayPort;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_price_gateway_port::RestaurantPriceGatewayPort;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_reservation_gateway_port::RestaurantReservationGatewayPort;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_takeaway_gateway_port::RestaurantTakeawayGatewayPort;
use crate::core::conversation::domain::model::intent::IntentId;

pub struct RestaurantConversationDependencies<'a> {
    pub opening_hours_port: &'a dyn RestaurantOpeningHoursGatewayPort,
    pub menu_port: &'a dyn RestaurantMenuGatewayPort,
    pub menu_dietary_port: &'a dyn RestaurantMenuDietaryGatewayPort,
    pub menu_item_details_port: &'a dyn RestaurantMenuItemDetailsGatewayPort,
    pub price_port: &'a dyn RestaurantPriceGatewayPort,
    pub location_port: &'a dyn RestaurantLocationGatewayPort,
    pub contact_port: &'a dyn RestaurantContactGatewayPort,
    pub payment_methods_port: &'a dyn RestaurantPaymentMethodsGatewayPort,
    pub takeaway_port: &'a dyn RestaurantTakeawayGatewayPort,
    pub event_port: &'a dyn RestaurantEventGatewayPort,
    pub facilities_port: &'a dyn RestaurantFacilitiesGatewayPort,
    pub accessibility_port: &'a dyn RestaurantAccessibilityGatewayPort,
    pub entertainment_port: &'a dyn RestaurantEntertainmentGatewayPort,
    pub reservation_port: &'a dyn RestaurantReservationGatewayPort,
}

/// Builds the [`IntentHandlerRegistry`] for the restaurant domain.
///
/// Each handler receives the smallest outbound capability it needs. The
/// registry stays the composition seam between `conversation` and `restaurant`.
pub struct RestaurantHandlerRegistryFactory;

impl RestaurantHandlerRegistryFactory {
    pub fn build<'a>(deps: RestaurantConversationDependencies<'a>) -> IntentHandlerRegistry<'a> {
        let RestaurantConversationDependencies {
            opening_hours_port,
            menu_port,
            menu_dietary_port,
            menu_item_details_port,
            price_port,
            location_port,
            contact_port,
            payment_methods_port,
            takeaway_port,
            event_port,
            facilities_port,
            accessibility_port,
            entertainment_port,
            reservation_port,
        } = deps;

        let handlers: Vec<Box<dyn IntentHandler + 'a>> = vec![
            Box::new(ReservationCreateIntentHandler::new(reservation_port)),
            Box::new(ReservationCancelIntentHandler::new(reservation_port)),
            Box::new(OpeningHoursIntentHandler::new(opening_hours_port)),
            Box::new(MenuItemDetailsIntentHandler::new(menu_item_details_port)),
            Box::new(AskMenuGeneralIntentHandler::new(menu_port)),
            Box::new(AskMenuDietaryIntentHandler::new(menu_dietary_port)),
            Box::new(AskLocationIntentHandler::new(location_port)),
            Box::new(AskContactIntentHandler::new(contact_port)),
            Box::new(AskPaymentMethodsIntentHandler::new(payment_methods_port)),
            Box::new(AskPriceIntentHandler::new(price_port)),
            Box::new(AskTakeawayDeliveryIntentHandler::new(takeaway_port)),
            Box::new(AskEventIntentHandler::new(event_port)),
            Box::new(AskFacilitiesIntentHandler::new(facilities_port)),
            Box::new(AskAccessibilityIntentHandler::new(accessibility_port)),
            Box::new(AskEntertainmentIntentHandler::new(entertainment_port)),
            Box::new(CheckReservationIntentHandler::new(reservation_port)),
            Box::new(StaticReplyIntentHandler::new(
                IntentId::Greeting,
                "intent.greeting.reply",
            )),
            Box::new(StaticReplyIntentHandler::new(
                IntentId::Thanks,
                "intent.thanks.reply",
            )),
            Box::new(StaticReplyIntentHandler::new(
                IntentId::Goodbye,
                "intent.goodbye.reply",
            )),
            Box::new(StaticReplyIntentHandler::new(
                IntentId::Unknown("unknown".to_string()),
                "intent.unknown.reply",
            )),
        ];
        IntentHandlerRegistry::new(handlers)
    }
}
