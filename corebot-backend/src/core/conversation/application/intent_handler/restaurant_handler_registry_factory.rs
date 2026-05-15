use std::sync::Arc;

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
use crate::core::conversation::application::port::outbound::restaurant_information_port::RestaurantInformationPort;
use crate::core::conversation::application::port::outbound::restaurant_reservation_port::RestaurantReservationPort;
use crate::core::conversation::domain::model::intent::IntentId;

pub struct RestaurantConversationDependencies<I, R>
where
    I: RestaurantInformationPort + Send + Sync + 'static,
    R: RestaurantReservationPort + Send + Sync + 'static,
{
    pub information_port: Arc<I>,
    pub reservation_port: Arc<R>,
}

/// Builds the [`IntentHandlerRegistry`] for the restaurant domain.
///
/// Each handler receives the smallest outbound capability it needs. The
/// registry stays the composition seam between `conversation` and `restaurant`.
pub struct RestaurantHandlerRegistryFactory;

impl RestaurantHandlerRegistryFactory {
    pub fn build<I, R>(deps: RestaurantConversationDependencies<I, R>) -> IntentHandlerRegistry
    where
        I: RestaurantInformationPort + Send + Sync + 'static,
        R: RestaurantReservationPort + Send + Sync + 'static,
    {
        let RestaurantConversationDependencies {
            information_port,
            reservation_port,
        } = deps;

        let handlers: Vec<Box<dyn IntentHandler>> = vec![
            Box::new(ReservationCreateIntentHandler::new(Arc::clone(
                &reservation_port,
            ))),
            Box::new(ReservationCancelIntentHandler),
            Box::new(OpeningHoursIntentHandler::new(Arc::clone(
                &information_port,
            ))),
            Box::new(MenuItemDetailsIntentHandler::new(Arc::clone(
                &information_port,
            ))),
            Box::new(AskMenuGeneralIntentHandler::new(Arc::clone(
                &information_port,
            ))),
            Box::new(AskMenuDietaryIntentHandler::new(Arc::clone(
                &information_port,
            ))),
            Box::new(AskLocationIntentHandler::new(Arc::clone(&information_port))),
            Box::new(AskContactIntentHandler::new(Arc::clone(&information_port))),
            Box::new(AskPaymentMethodsIntentHandler::new(Arc::clone(
                &information_port,
            ))),
            Box::new(AskPriceIntentHandler::new(Arc::clone(&information_port))),
            Box::new(AskTakeawayDeliveryIntentHandler::new(Arc::clone(
                &information_port,
            ))),
            Box::new(AskEventIntentHandler::new(Arc::clone(&information_port))),
            Box::new(AskFacilitiesIntentHandler::new(Arc::clone(
                &information_port,
            ))),
            Box::new(AskAccessibilityIntentHandler::new(Arc::clone(
                &information_port,
            ))),
            Box::new(AskEntertainmentIntentHandler::new(Arc::clone(
                &information_port,
            ))),
            Box::new(CheckReservationIntentHandler::new(Arc::clone(
                &reservation_port,
            ))),
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
