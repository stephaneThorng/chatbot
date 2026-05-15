use crate::core::conversation::application::intent_handler::{IntentHandler, IntentHandlerRegistry};
use crate::core::conversation::application::intent_handlers::ask_accessibility_handler::AskAccessibilityIntentHandler;
use crate::core::conversation::application::intent_handlers::ask_contact_handler::AskContactIntentHandler;
use crate::core::conversation::application::intent_handlers::ask_entertainment_handler::AskEntertainmentIntentHandler;
use crate::core::conversation::application::intent_handlers::ask_event_handler::AskEventIntentHandler;
use crate::core::conversation::application::intent_handlers::ask_facilities_handler::AskFacilitiesIntentHandler;
use crate::core::conversation::application::intent_handlers::ask_location_handler::AskLocationIntentHandler;
use crate::core::conversation::application::intent_handlers::ask_menu_dietary_handler::AskMenuDietaryIntentHandler;
use crate::core::conversation::application::intent_handlers::ask_menu_general_handler::AskMenuGeneralIntentHandler;
use crate::core::conversation::application::intent_handlers::ask_payment_methods_handler::AskPaymentMethodsIntentHandler;
use crate::core::conversation::application::intent_handlers::ask_price_handler::AskPriceIntentHandler;
use crate::core::conversation::application::intent_handlers::ask_takeaway_delivery_handler::AskTakeawayDeliveryIntentHandler;
use crate::core::conversation::application::intent_handlers::check_reservation_handler::CheckReservationIntentHandler;
use crate::core::conversation::application::intent_handlers::menu_item_details_handler::MenuItemDetailsIntentHandler;
use crate::core::conversation::application::intent_handlers::opening_hours_handler::OpeningHoursIntentHandler;
use crate::core::conversation::application::intent_handlers::reservation_cancel_handler::ReservationCancelIntentHandler;
use crate::core::conversation::application::intent_handlers::reservation_create_handler::ReservationCreateIntentHandler;
use crate::core::conversation::application::intent_handlers::static_reply_handler::StaticReplyIntentHandler;
use crate::core::conversation::application::port::outbound::restaurant_information_port::RestaurantInformationPort;
use crate::core::conversation::application::port::outbound::restaurant_reservation_port::RestaurantReservationPort;
use crate::core::conversation::domain::model::intent::IntentId;

pub struct RestaurantConversationDependencies<'a, I, R>
where
    I: RestaurantInformationPort + Send + Sync + ?Sized,
    R: RestaurantReservationPort + Send + Sync + ?Sized,
{
    pub information_port: &'a I,
    pub reservation_port: &'a R,
}

/// Builds the [`IntentHandlerRegistry`] for the restaurant domain.
///
/// Each handler receives the smallest outbound capability it needs. The
/// registry stays the composition seam between `conversation` and `restaurant`.
pub struct RestaurantHandlerRegistryFactory;

impl RestaurantHandlerRegistryFactory {
    pub fn build<'a, I, R>(
        deps: RestaurantConversationDependencies<'a, I, R>,
    ) -> IntentHandlerRegistry<'a>
    where
        I: RestaurantInformationPort + Send + Sync + ?Sized + 'a,
        R: RestaurantReservationPort + Send + Sync + ?Sized + 'a,
    {
        let RestaurantConversationDependencies {
            information_port,
            reservation_port,
        } = deps;

        let handlers: Vec<Box<dyn IntentHandler + 'a>> = vec![
            Box::new(ReservationCreateIntentHandler::new(reservation_port)),
            Box::new(ReservationCancelIntentHandler),
            Box::new(OpeningHoursIntentHandler::new(information_port)),
            Box::new(MenuItemDetailsIntentHandler::new(information_port)),
            Box::new(AskMenuGeneralIntentHandler::new(information_port)),
            Box::new(AskMenuDietaryIntentHandler::new(information_port)),
            Box::new(AskLocationIntentHandler::new(information_port)),
            Box::new(AskContactIntentHandler::new(information_port)),
            Box::new(AskPaymentMethodsIntentHandler::new(information_port)),
            Box::new(AskPriceIntentHandler::new(information_port)),
            Box::new(AskTakeawayDeliveryIntentHandler::new(information_port)),
            Box::new(AskEventIntentHandler::new(information_port)),
            Box::new(AskFacilitiesIntentHandler::new(information_port)),
            Box::new(AskAccessibilityIntentHandler::new(information_port)),
            Box::new(AskEntertainmentIntentHandler::new(information_port)),
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
