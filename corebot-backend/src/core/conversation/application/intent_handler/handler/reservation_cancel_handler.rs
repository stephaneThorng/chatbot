use crate::core::conversation::application::intent_handler::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult, WorkflowPostProcessResult,
};
use crate::core::conversation::application::port::outbound::restaurant::restaurant_availability_repository_port::RestaurantAvailabilityRepositoryPort;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_reservation_repository_port::RestaurantReservationRepositoryPort;
use crate::core::conversation::application::port::outbound::restaurant::reservation_queries::{
    ReservationCancelFailure, ReservationCancelQuery,
};
use crate::core::conversation::application::service::restaurant::{
    ConversationRestaurantReservationService,
};
use crate::core::conversation::application::util::workflow_slot_reader::ReservationCancelSlots;
use crate::core::conversation::domain::model::conversation::Conversation;
use crate::core::conversation::domain::model::intent::{
    IntentConfig, IntentId, IntentWorkflow, NluTask, WorkflowConfig, i18n_key,
};
use crate::core::conversation::domain::model::slot::{
    SlotConfig, SlotConstraint, SlotConstraintEntry, SlotName,
};

pub struct ReservationCancelIntentHandler<'a, R, A> {
    reservation_service: &'a ConversationRestaurantReservationService<R, A>,
}

impl<'a, R, A> ReservationCancelIntentHandler<'a, R, A> {
    pub fn new(reservation_service: &'a ConversationRestaurantReservationService<R, A>) -> Self {
        Self {
            reservation_service,
        }
    }
}

#[async_trait::async_trait]
impl<R, A> IntentHandler for ReservationCancelIntentHandler<'_, R, A>
where
    R: RestaurantReservationRepositoryPort + Send + Sync,
    A: RestaurantAvailabilityRepositoryPort + Send + Sync,
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
                starting_message: Some(i18n_key("workflow.reservation_cancel.starting.message")),
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
                reply: vec![
                    rust_i18n::t!(
                        "workflow.reservation_cancel.slot.reference.prompt",
                        locale = lang
                    )
                    .to_string(),
                ],
            };
        }

        match self
            .reservation_service
            .cancel_reservation(
                conversation.business_id,
                ReservationCancelQuery {
                    reference: slots.reference.clone(),
                    name: slots.name.clone(),
                    date: slots.date,
                },
            )
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
                    reply: vec![
                        rust_i18n::t!(
                            "intent.check_reservation.not_found.reply",
                            locale = lang,
                            reference = slots.reference
                        )
                        .to_string(),
                    ],
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::conversation::application::port::outbound::restaurant::restaurant_availability_repository_port::RestaurantAvailabilityRepositoryPort;
    use crate::core::conversation::application::port::outbound::restaurant::restaurant_business_info_repository_port::RestaurantBusinessInfoRepositoryPort;
    use crate::core::conversation::application::port::outbound::restaurant::restaurant_menu_repository_port::RestaurantMenuRepositoryPort;
    use crate::core::conversation::application::port::outbound::restaurant::restaurant_reservation_repository_port::RestaurantReservationRepositoryPort;
    use crate::core::conversation::application::service::restaurant::{
        ConversationRestaurantReservationService,
    };
    use crate::core::conversation::domain::conversation::Conversation;
    use crate::core::conversation::domain::domain_type::DomainType;
    use crate::core::conversation::domain::restaurant::model::{
        AmountComparator, BusinessFact, BusinessLocation, ContactChannel, EventSpace, Facility,
        MenuItem, OpeningHours, PaymentMethod, Reservation, ReservationDraft,
        ReservationSettings, RestaurantRepositoryError, TableType,
    };
    use crate::core::conversation::domain::model::slot::{SlotDataValue, SlotName};
    use chrono::{NaiveDate, NaiveTime, Weekday};
    use uuid::Uuid;

    #[derive(Clone)]
    struct StubRepository;

    #[async_trait::async_trait]
    impl RestaurantBusinessInfoRepositoryPort for StubRepository {
        async fn opening_hours(
            &self,
            _: Uuid,
        ) -> Result<Vec<OpeningHours>, RestaurantRepositoryError> {
            Ok(vec![])
        }
        async fn location(
            &self,
            _: Uuid,
        ) -> Result<Option<BusinessLocation>, RestaurantRepositoryError> {
            Ok(None)
        }
        async fn contact_channels(
            &self,
            _: Uuid,
        ) -> Result<Vec<ContactChannel>, RestaurantRepositoryError> {
            Ok(vec![])
        }
        async fn payment_methods(
            &self,
            _: Uuid,
        ) -> Result<Vec<PaymentMethod>, RestaurantRepositoryError> {
            Ok(vec![])
        }
        async fn facilities(&self, _: Uuid) -> Result<Vec<Facility>, RestaurantRepositoryError> {
            Ok(vec![])
        }
        async fn facts(
            &self,
            _: Uuid,
            _: &str,
        ) -> Result<Vec<BusinessFact>, RestaurantRepositoryError> {
            Ok(vec![])
        }
        async fn event_spaces(
            &self,
            _: Uuid,
        ) -> Result<Vec<EventSpace>, RestaurantRepositoryError> {
            Ok(vec![])
        }
    }

    #[async_trait::async_trait]
    impl RestaurantMenuRepositoryPort for StubRepository {
        async fn menu_items(
            &self,
            _: Uuid,
            _: &str,
        ) -> Result<Vec<MenuItem>, RestaurantRepositoryError> {
            Ok(vec![])
        }
        async fn menu_items_by_price(
            &self,
            _: Uuid,
            _: &str,
            _: &AmountComparator,
        ) -> Result<Vec<MenuItem>, RestaurantRepositoryError> {
            Ok(vec![])
        }
    }

    #[async_trait::async_trait]
    impl RestaurantReservationRepositoryPort for StubRepository {
        async fn next_reference_index(&self, _: Uuid) -> Result<i64, RestaurantRepositoryError> {
            Ok(1)
        }
        async fn create_reservation(
            &self,
            _: Uuid,
            _: ReservationDraft,
        ) -> Result<Reservation, RestaurantRepositoryError> {
            unreachable!("create_reservation is not used in cancel handler tests")
        }
        async fn find_by_reference(
            &self,
            _: Uuid,
            _: &str,
        ) -> Result<Option<Reservation>, RestaurantRepositoryError> {
            Ok(None)
        }
        async fn find_by_name(
            &self,
            _: Uuid,
            _: &str,
        ) -> Result<Vec<Reservation>, RestaurantRepositoryError> {
            Ok(vec![])
        }
        async fn cancel_by_reference(
            &self,
            _: Uuid,
            reference: &str,
        ) -> Result<Option<Reservation>, RestaurantRepositoryError> {
            Ok(Some(Reservation {
                reference: reference.to_string(),
                name: "Alice".to_string(),
                date: NaiveDate::from_ymd_opt(2099, 6, 12).unwrap(),
                time: NaiveTime::from_hms_opt(19, 0, 0).unwrap(),
                people_count: 2,
            }))
        }
    }

    #[async_trait::async_trait]
    impl RestaurantAvailabilityRepositoryPort for StubRepository {
        async fn reservation_settings(
            &self,
            _: Uuid,
        ) -> Result<ReservationSettings, RestaurantRepositoryError> {
            Ok(ReservationSettings {
                slot_minutes: 120,
                max_lookup_days: 7,
            })
        }
        async fn table_types(&self, _: Uuid) -> Result<Vec<TableType>, RestaurantRepositoryError> {
            Ok(vec![TableType {
                capacity: 4,
                count: 1,
            }])
        }
        async fn opening_hours(
            &self,
            _: Uuid,
        ) -> Result<Vec<OpeningHours>, RestaurantRepositoryError> {
            Ok(vec![OpeningHours {
                day_of_week: Weekday::Fri,
                opens_at: NaiveTime::from_hms_opt(11, 0, 0).unwrap(),
                closes_at: NaiveTime::from_hms_opt(23, 0, 0).unwrap(),
                is_closed: false,
            }])
        }
        async fn is_closed_at(
            &self,
            _: Uuid,
            _: NaiveDate,
            _: NaiveTime,
            _: u32,
        ) -> Result<bool, RestaurantRepositoryError> {
            Ok(false)
        }
        async fn reservations_near(
            &self,
            _: Uuid,
            _: NaiveDate,
        ) -> Result<Vec<Reservation>, RestaurantRepositoryError> {
            Ok(vec![])
        }
    }

    fn handler() -> ReservationCancelIntentHandler<'static, StubRepository, StubRepository> {
        let repository = StubRepository;
        let reservation_repository = repository.clone();
        let availability_repository = repository;
        let leaked = Box::leak(Box::new(ConversationRestaurantReservationService::new(
            reservation_repository,
            availability_repository,
        )));
        ReservationCancelIntentHandler::new(leaked)
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

        assert_eq!(
            result.reply,
            vec!["Your reservation cancellation is confirmed.".to_string()]
        );
        assert!(result.updated_conversation.is_idle());
        assert_eq!(
            result.updated_conversation.last_reservation_reference(),
            Some("REST-NEW123")
        );
    }
}
