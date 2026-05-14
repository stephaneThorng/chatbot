use rust_i18n::t;
use std::sync::Arc;

use crate::core::conversation::application::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant_queries::ReservationLookupQuery;
use crate::core::conversation::application::port::outbound::restaurant_reservation_port::RestaurantReservationPort;
use crate::core::conversation::domain::model::intent::{IntentId, IntentKind, IntentPolicy};
use crate::core::conversation::domain::model::slot::EntityType;

pub struct CheckReservationIntentHandler<P: RestaurantReservationPort + ?Sized> {
    reservation_port: Arc<P>,
}

impl<P: RestaurantReservationPort + ?Sized> CheckReservationIntentHandler<P> {
    pub fn new(reservation_port: Arc<P>) -> Self {
        Self { reservation_port }
    }
}

impl<P: RestaurantReservationPort + Send + Sync + ?Sized> IntentHandler
    for CheckReservationIntentHandler<P>
{
    fn intent(&self) -> IntentId {
        IntentId::CheckReservation
    }

    fn policy(&self) -> IntentPolicy {
        IntentPolicy {
            id: self.intent(),
            kind: IntentKind::Informational,
            nlu_task: None,
            workflow_slots: vec![],
            confirmation_prompt: None,
            completion_response: None,
        }
    }

    fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let lang = input.conversation.lang.as_str();
        let reference = self.lookup_entity_value(&input, EntityType::ReservationReference);
        let name = self
            .lookup_entity_value(&input, EntityType::Person)
            .or_else(|| input.conversation.known_customer_name());
        let raw = self
            .reservation_port
            .check_reservation(ReservationLookupQuery {
                reference: reference.map(str::to_string),
                name: name.map(str::to_string),
            });

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
            reply,
            handled_intent: self.intent(),
        }
    }
}
