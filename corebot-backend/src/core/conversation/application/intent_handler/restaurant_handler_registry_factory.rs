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
use crate::core::conversation::application::port::outbound::restaurant::restaurant_availability_repository_port::RestaurantAvailabilityRepositoryPort;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_business_info_repository_port::RestaurantBusinessInfoRepositoryPort;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_menu_repository_port::RestaurantMenuRepositoryPort;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_reservation_repository_port::RestaurantReservationRepositoryPort;
use crate::core::conversation::application::service::restaurant::{
    ConversationRestaurantMenuService, ConversationRestaurantReservationService,
};
use crate::core::conversation::domain::model::intent::IntentId;

pub struct RestaurantConversationDependencies<'a, B, M, R, A> {
    pub business_info_repository: &'a B,
    pub menu_service: &'a ConversationRestaurantMenuService<M>,
    pub reservation_service: &'a ConversationRestaurantReservationService<R, A>,
}

/// Builds the [`IntentHandlerRegistry`] for the restaurant domain.
///
/// Each handler receives the smallest outbound capability it needs. The
/// registry stays the composition seam between `conversation` and `restaurant`.
pub struct RestaurantHandlerRegistryFactory;

impl RestaurantHandlerRegistryFactory {
    pub fn build<'a, B, M, R, A>(
        deps: RestaurantConversationDependencies<'a, B, M, R, A>,
    ) -> IntentHandlerRegistry<'a>
    where
        B: RestaurantBusinessInfoRepositoryPort + Send + Sync,
        M: RestaurantMenuRepositoryPort + Send + Sync,
        R: RestaurantReservationRepositoryPort + Send + Sync,
        A: RestaurantAvailabilityRepositoryPort + Send + Sync,
    {
        let RestaurantConversationDependencies {
            business_info_repository,
            menu_service,
            reservation_service,
        } = deps;

        let handlers: Vec<Box<dyn IntentHandler + 'a>> = vec![
            Box::new(ReservationCreateIntentHandler::new(reservation_service)),
            Box::new(ReservationCancelIntentHandler::new(reservation_service)),
            Box::new(OpeningHoursIntentHandler::new(business_info_repository)),
            Box::new(MenuItemDetailsIntentHandler::new(menu_service)),
            Box::new(AskMenuGeneralIntentHandler::new(menu_service)),
            Box::new(AskMenuDietaryIntentHandler::new(menu_service)),
            Box::new(AskLocationIntentHandler::new(business_info_repository)),
            Box::new(AskContactIntentHandler::new(business_info_repository)),
            Box::new(AskPaymentMethodsIntentHandler::new(
                business_info_repository,
            )),
            Box::new(AskPriceIntentHandler::new(menu_service, business_info_repository)),
            Box::new(AskTakeawayDeliveryIntentHandler::new(
                business_info_repository,
            )),
            Box::new(AskEventIntentHandler::new(business_info_repository)),
            Box::new(AskFacilitiesIntentHandler::new(business_info_repository)),
            Box::new(AskAccessibilityIntentHandler::new(business_info_repository)),
            Box::new(AskEntertainmentIntentHandler::new(business_info_repository)),
            Box::new(CheckReservationIntentHandler::new(reservation_service)),
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
