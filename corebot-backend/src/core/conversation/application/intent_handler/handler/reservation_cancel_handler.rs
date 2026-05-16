use crate::core::conversation::application::intent_handler::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult, WorkflowPostProcessResult,
};
use crate::core::conversation::application::port::outbound::restaurant::reservation_queries::{
    ReservationCancelFailure, ReservationCancelQuery,
};
use crate::core::conversation::application::port::outbound::restaurant::restaurant_reservation_gateway_port::RestaurantReservationGatewayPort;
use crate::core::conversation::application::util::workflow_slot_reader::ReservationCancelSlots;
use crate::core::conversation::domain::model::conversation::Conversation;
use crate::core::conversation::domain::model::intent::{
    IntentConfig, IntentId, IntentWorkflow, NluTask, WorkflowConfig, i18n_key,
};
use crate::core::conversation::domain::model::slot::{
    SlotConfig, SlotConstraint, SlotConstraintEntry, SlotName,
};

pub struct ReservationCancelIntentHandler<'a, P: RestaurantReservationGatewayPort + ?Sized> {
    reservation_gateway_port: &'a P,
}

impl<'a, P: RestaurantReservationGatewayPort + ?Sized> ReservationCancelIntentHandler<'a, P> {
    pub fn new(reservation_port: &'a P) -> Self {
        Self {
            reservation_gateway_port: reservation_port,
        }
    }
}

#[async_trait::async_trait]
impl<P: RestaurantReservationGatewayPort + Send + Sync + ?Sized> IntentHandler
    for ReservationCancelIntentHandler<'_, P>
{
    fn intent(&self) -> IntentId {
        IntentId::ReservationCancel
    }

    fn config(&self) -> IntentConfig {
        IntentConfig {
            id: self.intent(),
            workflow: IntentWorkflow::Workflow(WorkflowConfig {
                nlu_task: Some(NluTask::ReservationCancel),
                slots: vec![
                    SlotConfig {
                        name: SlotName::Reference,
                        required: true,
                        prompt: i18n_key("workflow.reservation_cancel.slot.reference.prompt"),
                        constraints: vec![],
                    },
                    SlotConfig {
                        name: SlotName::Name,
                        required: false,
                        prompt: i18n_key("workflow.reservation_cancel.slot.name.prompt"),
                        constraints: vec![],
                    },
                    SlotConfig {
                        name: SlotName::Date,
                        required: false,
                        prompt: i18n_key("workflow.reservation_cancel.slot.date.prompt"),
                        constraints: vec![SlotConstraintEntry::new(SlotConstraint::FutureDate)],
                    },
                ],
                starting_message: None,
                confirmation_prompt: Some(i18n_key(
                    "workflow.reservation_cancel.confirmation.prompt",
                )),
                completion_response: Some(i18n_key(
                    "workflow.reservation_cancel.completion.success",
                )),
            }),
        }
    }

    async fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        self.handle_workflow(input).await
    }

    fn confirmation_prompt(
        &self,
        _workflow_cfg: &WorkflowConfig,
        conversation: &Conversation,
    ) -> String {
        let slots = ReservationCancelSlots::from_conversation(conversation);
        let lang = conversation.lang.as_str();
        if let Some(date) = slots.date {
            return rust_i18n::t!(
                "workflow.reservation_cancel.confirmation.prompt",
                locale = lang
            )
            .to_string()
                + "\n"
                + &format!("Reference: {} | Date: {}", slots.reference, date);
        }
        rust_i18n::t!(
            "workflow.reservation_cancel.confirmation.prompt",
            locale = lang
        )
        .to_string()
    }

    async fn post_process(
        &self,
        lang: &str,
        mut conversation: Conversation,
    ) -> WorkflowPostProcessResult {
        let slots = ReservationCancelSlots::from_conversation(&conversation);
        if slots.reference.is_empty() {
            conversation.clear_workflow_slot(SlotName::Reference);
            return WorkflowPostProcessResult::Failed {
                updated_conversation: conversation,
                reply: rust_i18n::t!(
                    "workflow.reservation_cancel.slot.reference.prompt",
                    locale = lang
                )
                .to_string(),
            };
        }

        match self
            .reservation_gateway_port
            .cancel_reservation(ReservationCancelQuery {
                reference: slots.reference.clone(),
                name: slots.name.clone(),
                date: slots.date,
            })
            .await
        {
            Ok(cancelled) => {
                let reference = cancelled
                    .strip_prefix("cancelled:")
                    .unwrap_or(cancelled.as_str())
                    .to_string();
                conversation.remember_reservation_reference(reference);
                if let Some(name) = slots.name {
                    conversation.remember_customer_name(name);
                }
                WorkflowPostProcessResult::Succeeded {
                    updated_conversation: conversation,
                    reply: None,
                }
            }
            Err(ReservationCancelFailure::NotFound) => {
                conversation.clear_workflow_slot(SlotName::Reference);
                WorkflowPostProcessResult::Failed {
                    updated_conversation: conversation,
                    reply: rust_i18n::t!(
                        "intent.check_reservation.not_found.reply",
                        locale = lang,
                        reference = slots.reference
                    )
                    .to_string(),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::conversation::application::port::outbound::restaurant::reservation_queries::ReservationLookupQuery;
    use crate::core::conversation::domain::conversation::Conversation;
    use crate::core::conversation::domain::domain_type::DomainType;
    use crate::core::conversation::domain::model::slot::{SlotDataValue, SlotName};

    struct StubReservationPort;

    #[async_trait::async_trait]
    impl RestaurantReservationGatewayPort for StubReservationPort {
        async fn create_reservation(
            &self,
            _: crate::core::conversation::application::port::outbound::restaurant::reservation_queries::ReservationCreateQuery,
        ) -> Result<String, crate::core::conversation::application::port::outbound::restaurant::reservation_queries::ReservationFailure>{
            Ok("created:REST-NEW123".to_string())
        }

        async fn cancel_reservation(
            &self,
            _: ReservationCancelQuery,
        ) -> Result<String, ReservationCancelFailure> {
            Ok("cancelled:REST-NEW123".to_string())
        }

        async fn check_reservation(&self, _: ReservationLookupQuery) -> String {
            "no_reference_or_name:".to_string()
        }
    }

    fn handler() -> ReservationCancelIntentHandler<'static, StubReservationPort> {
        static STUB_RESERVATION_PORT: StubReservationPort = StubReservationPort;
        ReservationCancelIntentHandler::new(&STUB_RESERVATION_PORT)
    }

    fn handle(conversation: Conversation, intent: IntentId, text: &str) -> StateHandlerResult {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test runtime should be created")
            .block_on(handler().handle(IntentHandlerInput {
                conversation,
                analysis_intent: &intent,
                text,
                analysis_entities: &[],
            }))
    }

    #[test]
    fn confirm_cancellation_completes_workflow() {
        let conversation = Conversation::new(DomainType::Restaurant)
            .into_started_workflow(&handler().config())
            .unwrap()
            .into_workflow_slot(
                SlotName::Reference,
                SlotDataValue::Text("REST-NEW123".to_string()),
            )
            .unwrap();

        let result = handle(conversation, IntentId::Affirmative, "");

        assert_eq!(result.reply, "Your reservation cancellation is confirmed.");
        assert!(result.updated_conversation.is_idle());
        assert_eq!(
            result.updated_conversation.last_reservation_reference(),
            Some("REST-NEW123")
        );
    }
}
