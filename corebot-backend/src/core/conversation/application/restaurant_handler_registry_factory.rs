use std::sync::Arc;

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
use crate::core::conversation::application::port::outbound::domain_gateway_port::DomainGatewayPort;
use crate::core::conversation::domain::date_resolver::DateResolver;
use crate::core::conversation::domain::model::intent::IntentId;

/// Builds the [`IntentHandlerRegistry`] for the restaurant domain.
///
/// Accepts a shared gateway and date resolver so all handlers share the same
/// backing data without extra allocation. Adding a new handler only requires
/// registering it here — the processor and use case stay unchanged.
pub struct RestaurantHandlerRegistryFactory;

impl RestaurantHandlerRegistryFactory {
    pub fn build<D>(
        gateway: Arc<D>,
        date_resolver: Arc<dyn DateResolver>,
    ) -> IntentHandlerRegistry
    where
        D: DomainGatewayPort + Send + Sync + 'static,
    {
        let handlers: Vec<Box<dyn IntentHandler>> = vec![
            Box::new(ReservationCreateIntentHandler::new(date_resolver)),
            Box::new(ReservationCancelIntentHandler),
            Box::new(OpeningHoursIntentHandler::new(Arc::clone(&gateway))),
            Box::new(MenuItemDetailsIntentHandler::new(Arc::clone(&gateway))),
            Box::new(AskMenuGeneralIntentHandler::new(Arc::clone(&gateway))),
            Box::new(AskMenuDietaryIntentHandler::new(Arc::clone(&gateway))),
            Box::new(AskLocationIntentHandler::new(Arc::clone(&gateway))),
            Box::new(AskContactIntentHandler::new(Arc::clone(&gateway))),
            Box::new(AskPaymentMethodsIntentHandler::new(Arc::clone(&gateway))),
            Box::new(AskPriceIntentHandler::new(Arc::clone(&gateway))),
            Box::new(AskTakeawayDeliveryIntentHandler::new(Arc::clone(&gateway))),
            Box::new(AskEventIntentHandler::new(Arc::clone(&gateway))),
            Box::new(AskFacilitiesIntentHandler::new(Arc::clone(&gateway))),
            Box::new(AskAccessibilityIntentHandler::new(Arc::clone(&gateway))),
            Box::new(AskEntertainmentIntentHandler::new(Arc::clone(&gateway))),
            Box::new(CheckReservationIntentHandler::new(Arc::clone(&gateway))),
            Box::new(StaticReplyIntentHandler::new(IntentId::Greeting, "intent.greeting.reply")),
            Box::new(StaticReplyIntentHandler::new(IntentId::Thanks, "intent.thanks.reply")),
            Box::new(StaticReplyIntentHandler::new(IntentId::Goodbye, "intent.goodbye.reply")),
            Box::new(StaticReplyIntentHandler::new(IntentId::Unknown("unknown".to_string()), "intent.unknown.reply")),
        ];
        IntentHandlerRegistry::new(handlers)
    }
}

