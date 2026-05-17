use rust_i18n::t;

use crate::core::conversation::application::intent_handler::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant::restaurant_availability_repository_port::RestaurantAvailabilityRepositoryPort;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_reservation_repository_port::RestaurantReservationRepositoryPort;
use crate::core::conversation::application::port::outbound::restaurant::reservation_queries::ReservationLookupQuery;
use crate::core::conversation::application::service::restaurant::{
    ConversationRestaurantReservationService,
};
use crate::core::conversation::domain::model::intent::{IntentConfig, IntentId, IntentWorkflow};

pub struct CheckReservationIntentHandler<'a, R, A> {
    reservation_service: &'a ConversationRestaurantReservationService<R, A>,
}

impl<'a, R, A> CheckReservationIntentHandler<'a, R, A> {
    pub fn new(reservation_service: &'a ConversationRestaurantReservationService<R, A>) -> Self {
        Self {
            reservation_service,
        }
    }
}

#[async_trait::async_trait]
impl<R, A> IntentHandler for CheckReservationIntentHandler<'_, R, A>
where
    R: RestaurantReservationRepositoryPort + Send + Sync,
    A: RestaurantAvailabilityRepositoryPort + Send + Sync,
{
    fn intent(&self) -> IntentId {
        IntentId::CheckReservation
    }

    fn config(&self) -> IntentConfig {
        IntentConfig {
            id: self.intent(),
            workflow: IntentWorkflow::Informational,
        }
    }

    async fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let lang = input.conversation.lang.as_str();
        let reference = self.lookup_entity_value(&input, "reservation_reference");
        let name = self
            .lookup_entity_value(&input, "person")
            .or_else(|| input.conversation.known_customer_name());
        let raw = self
            .reservation_service
            .check_reservation(
                input.conversation.business_id,
                ReservationLookupQuery {
                    reference: reference.map(str::to_string),
                    name: name.map(str::to_string),
                },
            )
            .await;

        let reply = if let Some(payload) = raw.strip_prefix("found:") {
            let parts: Vec<&str> = payload.splitn(5, '|').collect();
            let r#ref = parts.first().copied().unwrap_or("");
            let name = parts.get(1).copied().unwrap_or("");
            let date = parts.get(2).copied().unwrap_or("");
            let time = parts.get(3).copied().unwrap_or("");
            let people = parts.get(4).copied().unwrap_or("");
            t!(
                "intent.check_reservation.found.reply",
                locale = lang,
                reference = r#ref,
                name = name,
                date = date,
                time = time,
                people = people
            )
            .to_string()
        } else if let Some(payload) = raw.strip_prefix("listed:") {
            let parts: Vec<&str> = payload.splitn(2, '|').collect();
            let name = parts.first().copied().unwrap_or("");
            let items = parts
                .get(1)
                .copied()
                .unwrap_or("")
                .split(';')
                .filter(|value| !value.is_empty())
                .map(|value| {
                    let fields: Vec<&str> = value.splitn(4, '~').collect();
                    let reference = fields.first().copied().unwrap_or("");
                    let date = fields.get(1).copied().unwrap_or("");
                    let time = fields.get(2).copied().unwrap_or("");
                    let people = fields.get(3).copied().unwrap_or("");
                    format!("{reference} on {date} at {time} for {people} people")
                })
                .collect::<Vec<_>>()
                .join(", ");
            t!(
                "intent.check_reservation.list.reply",
                locale = lang,
                name = name,
                reservations = items
            )
            .to_string()
        } else if let Some(name) = raw.strip_prefix("name_not_found:") {
            t!(
                "intent.check_reservation.name_not_found.reply",
                locale = lang,
                name = name
            )
            .to_string()
        } else if let Some(r) = raw.strip_prefix("not_found:") {
            t!(
                "intent.check_reservation.not_found.reply",
                locale = lang,
                reference = r
            )
            .to_string()
        } else {
            t!("intent.check_reservation.reply", locale = lang).to_string()
        };

        StateHandlerResult {
            updated_conversation: input.conversation,
            reply: vec![reply],
            handled_intent: self.intent(),
        }
    }
}
