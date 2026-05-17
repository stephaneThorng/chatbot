use crate::core::conversation::application::intent_handler::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult, WorkflowPostProcessResult,
};
use crate::core::conversation::application::port::outbound::restaurant::restaurant_availability_repository_port::RestaurantAvailabilityRepositoryPort;
use crate::core::conversation::application::port::outbound::restaurant::restaurant_reservation_repository_port::RestaurantReservationRepositoryPort;
use crate::core::conversation::application::port::outbound::restaurant::reservation_queries::{
    ReservationCreateQuery, ReservationFailure,
};
use crate::core::conversation::application::service::restaurant::{
    ConversationRestaurantReservationService,
};
use crate::core::conversation::application::util::reservation_create_presenter;
use crate::core::conversation::application::util::workflow_slot_reader::ReservationCreateSlots;
use crate::core::conversation::domain::model::conversation::Conversation;
use crate::core::conversation::domain::model::intent::{
    IntentConfig, IntentId, IntentWorkflow, NluTask, WorkflowConfig, i18n_key,
};
use crate::core::conversation::domain::model::slot::{
    SlotConfig, SlotConstraint, SlotConstraintEntry, SlotName,
};

pub struct ReservationCreateIntentHandler<'a, R, A> {
    reservation_service: &'a ConversationRestaurantReservationService<R, A>,
}

impl<'a, R, A> ReservationCreateIntentHandler<'a, R, A>
where
    R: RestaurantReservationRepositoryPort + Send + Sync,
    A: RestaurantAvailabilityRepositoryPort + Send + Sync,
{
    pub fn new(reservation_service: &'a ConversationRestaurantReservationService<R, A>) -> Self {
        Self {
            reservation_service,
        }
    }

    #[cfg(test)]
    pub fn handle_blocking(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test runtime should be created")
            .block_on(<Self as IntentHandler>::handle(self, input))
    }
}

#[async_trait::async_trait]
impl<R, A> IntentHandler for ReservationCreateIntentHandler<'_, R, A>
where
    R: RestaurantReservationRepositoryPort + Send + Sync,
    A: RestaurantAvailabilityRepositoryPort + Send + Sync,
{
    fn intent(&self) -> IntentId {
        IntentId::ReservationCreate
    }

    fn config(&self) -> IntentConfig {
        IntentConfig {
            id: self.intent(),
            workflow: IntentWorkflow::Workflow(WorkflowConfig {
                nlu_task: Some(NluTask::ReservationCreate),
                slots: vec![
                    SlotConfig {
                        name: SlotName::Name,
                        required: true,
                        prompt: i18n_key("workflow.reservation_create.slot.name.prompt"),
                        constraints: vec![SlotConstraintEntry::new(SlotConstraint::TextMaxLen(
                            100,
                        ))],
                    },
                    SlotConfig {
                        name: SlotName::Date,
                        required: true,
                        prompt: i18n_key("workflow.reservation_create.slot.date.prompt"),
                        constraints: vec![SlotConstraintEntry::with_error_key(
                            SlotConstraint::FutureDate,
                            "workflow.reservation_create.past_date.error",
                        )],
                    },
                    SlotConfig {
                        name: SlotName::Time,
                        required: true,
                        prompt: i18n_key("workflow.reservation_create.slot.time.prompt"),
                        constraints: vec![],
                    },
                    SlotConfig {
                        name: SlotName::People,
                        required: true,
                        prompt: i18n_key("workflow.reservation_create.slot.people.prompt"),
                        constraints: vec![SlotConstraintEntry::new(SlotConstraint::NumberRange(
                            1, 20,
                        ))],
                    },
                ],
                starting_message: Some(i18n_key("workflow.reservation_create.starting.message")),
                confirmation_prompt: Some(i18n_key(
                    "workflow.reservation_create.confirmation.prompt",
                )),
                completion_response: Some(i18n_key(
                    "workflow.reservation_create.completion.success",
                )),
            }),
        }
    }

    async fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        self.handle_workflow(input).await
    }

    fn negative_prompt(&self, lang: &str) -> String {
        rust_i18n::t!("workflow.reservation_create.update.prompt", locale = lang).to_string()
    }

    fn confirmation_prompt(
        &self,
        _workflow_cfg: &crate::core::conversation::domain::model::intent::WorkflowConfig,
        conversation: &Conversation,
    ) -> String {
        let slots = ReservationCreateSlots::from_conversation(conversation);
        reservation_create_presenter::confirmation_summary(&slots, conversation.lang.as_str())
    }

    async fn post_process(
        &self,
        lang: &str,
        mut conversation: Conversation,
    ) -> WorkflowPostProcessResult {
        let slots = ReservationCreateSlots::from_conversation(&conversation);

        let Some(date) = slots.date else {
            conversation.clear_workflow_slot(SlotName::Date);
            conversation.clear_workflow_slot(SlotName::Time);
            return WorkflowPostProcessResult::Failed {
                updated_conversation: conversation,
                reply: vec![
                    rust_i18n::t!(
                        "workflow.reservation_create.slot.date.prompt",
                        locale = lang
                    )
                    .to_string(),
                ],
            };
        };

        let Some(time) = slots.time else {
            conversation.clear_workflow_slot(SlotName::Time);
            return WorkflowPostProcessResult::Failed {
                updated_conversation: conversation,
                reply: vec![
                    rust_i18n::t!(
                        "workflow.reservation_create.time_invalid.error",
                        locale = lang
                    )
                    .to_string(),
                ],
            };
        };

        match self
            .reservation_service
            .create_reservation(
                conversation.business_id,
                ReservationCreateQuery {
                    name: slots.name.clone(),
                    date,
                    time,
                    people_count: slots.people_count,
                },
            )
            .await
        {
            Ok(creation) => {
                let reference = creation
                    .strip_prefix("created:")
                    .unwrap_or(creation.as_str())
                    .to_string();
                conversation.remember_customer_name(slots.name.clone());
                conversation.remember_reservation_reference(reference.clone());
                WorkflowPostProcessResult::Succeeded {
                    updated_conversation: conversation,
                    reply: Some(vec![reservation_create_presenter::completion_summary(
                        &slots, &reference, lang,
                    )]),
                }
            }
            Err(ReservationFailure::RestaurantClosed) => {
                conversation.clear_workflow_slot(SlotName::Date);
                conversation.clear_workflow_slot(SlotName::Time);
                WorkflowPostProcessResult::Failed {
                    updated_conversation: conversation,
                    reply: vec![
                        rust_i18n::t!(
                            "workflow.reservation_create.closed.error",
                            locale = lang
                        )
                        .to_string(),
                    ],
                }
            }
            Err(ReservationFailure::NoAvailability { next_slot }) => {
                conversation.clear_workflow_slot(SlotName::Date);
                conversation.clear_workflow_slot(SlotName::Time);
                let reply = match next_slot {
                    Some(suggestion) => rust_i18n::t!(
                        "workflow.reservation_create.no_availability_with_suggestion.error",
                        locale = lang,
                        next_slot = suggestion
                    )
                    .to_string(),
                    None => rust_i18n::t!(
                        "workflow.reservation_create.no_availability.error",
                        locale = lang
                    )
                    .to_string(),
                };
                WorkflowPostProcessResult::Failed {
                    updated_conversation: conversation,
                    reply: vec![reply],
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::conversation::application::dto::nlu_analysis_result::NluEntityResult;
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

    #[derive(Clone, Copy)]
    enum RepositoryMode {
        Success,
        NoAvailability,
        Closed,
    }

    #[derive(Clone)]
    struct StubRepository {
        mode: RepositoryMode,
    }

    impl StubRepository {
        fn new(mode: RepositoryMode) -> Self {
            Self { mode }
        }
    }

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
            reservation: ReservationDraft,
        ) -> Result<Reservation, RestaurantRepositoryError> {
            Ok(Reservation {
                reference: reservation.reference,
                name: reservation.name,
                date: reservation.date,
                time: reservation.time,
                people_count: reservation.people_count,
            })
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
            _: &str,
        ) -> Result<Option<Reservation>, RestaurantRepositoryError> {
            Ok(None)
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
            Ok([
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
                Weekday::Sat,
                Weekday::Sun,
            ]
            .into_iter()
            .map(|day_of_week| OpeningHours {
                day_of_week,
                opens_at: NaiveTime::from_hms_opt(11, 0, 0).unwrap(),
                closes_at: NaiveTime::from_hms_opt(23, 0, 0).unwrap(),
                is_closed: false,
            })
            .collect())
        }

        async fn is_closed_at(
            &self,
            _: Uuid,
            _: NaiveDate,
            _: NaiveTime,
            _: u32,
        ) -> Result<bool, RestaurantRepositoryError> {
            Ok(matches!(self.mode, RepositoryMode::Closed))
        }

        async fn reservations_near(
            &self,
            _: Uuid,
            date: NaiveDate,
        ) -> Result<Vec<Reservation>, RestaurantRepositoryError> {
            if matches!(self.mode, RepositoryMode::NoAvailability) {
                return Ok(vec![Reservation {
                    reference: "REST-EXISTING".to_string(),
                    name: "Booked".to_string(),
                    date,
                    time: NaiveTime::from_hms_opt(19, 0, 0).unwrap(),
                    people_count: 4,
                }]);
            }
            Ok(vec![])
        }
    }

    fn entity(entity_label: &'static str, value: &str) -> NluEntityResult {
        NluEntityResult {
            entity_label: entity_label.to_string(),
            value: value.to_string(),
            raw_value: value.to_string(),
            start: 0,
            end: value.len(),
            confidence: 1.0,
        }
    }

    fn build_service(
        mode: RepositoryMode,
    ) -> &'static ConversationRestaurantReservationService<StubRepository, StubRepository> {
        let repository = StubRepository::new(mode);
        let reservation_repository = repository.clone();
        let availability_repository = repository;
        let service = Box::leak(Box::new(ConversationRestaurantReservationService::new(
            reservation_repository,
            availability_repository,
        )));
        service
    }

    fn handler() -> ReservationCreateIntentHandler<'static, StubRepository, StubRepository> {
        let reservation_service = build_service(RepositoryMode::Success);
        ReservationCreateIntentHandler::new(reservation_service)
    }

    fn handle(
        conversation: Conversation,
        intent: IntentId,
        text: &str,
        entities: Vec<NluEntityResult>,
    ) -> StateHandlerResult {
        handler().handle_blocking(IntentHandlerInput {
            conversation,
            analysis_intent: &intent,
            text,
            analysis_entities: &entities,
        })
    }

    fn reply_text(reply: &[String]) -> String {
        reply.join("\n")
    }

    #[test]
    fn idle_workflow_prompts_for_first_missing_slot() {
        let conversation = Conversation::new(DomainType::Restaurant);
        let result = handle(conversation, IntentId::ReservationCreate, "", vec![]);
        assert!(reply_text(&result.reply).ends_with("What name should I use for the reservation?"));
        assert!(result.updated_conversation.has_active_workflow());
    }

    #[test]
    fn missing_slots_are_filled_from_entities() {
        let conversation = Conversation::new(DomainType::Restaurant);
        let result = handle(
            conversation,
            IntentId::ReservationCreate,
            "",
            vec![
                entity("person", "Alice"),
                entity("date", "2099-06-12"),
                entity("time", "7pm"),
            ],
        );
        let workflow = result.updated_conversation.active_workflow().unwrap();
        assert_eq!(
            workflow.slot_value(SlotName::Name),
            Some(&SlotDataValue::Text("Alice".to_string()))
        );
        assert!(reply_text(&result.reply).ends_with("For how many people?"));
    }

    #[test]
    fn filled_workflow_asks_for_confirmation() {
        let conversation = Conversation::new(DomainType::Restaurant);
        let result = handle(
            conversation,
            IntentId::ReservationCreate,
            "",
            vec![
                entity("person", "Alice"),
                entity("date", "2099-06-12"),
                entity("time", "7pm"),
                entity("people_count", "4"),
            ],
        );
        assert!(reply_text(&result.reply).contains("Alice"));
        assert!(reply_text(&result.reply).contains("19:00"));
        assert!(reply_text(&result.reply).contains("4 people"));
        assert!(reply_text(&result.reply).contains("Do you confirm"));
    }

    #[test]
    fn bare_numeric_reply_fills_people_slot_when_it_is_next() {
        let conversation = handle(
            Conversation::new(DomainType::Restaurant),
            IntentId::ReservationCreate,
            "",
            vec![
                entity("person", "Alice"),
                entity("date", "2099-06-12"),
                entity("time", "7pm"),
            ],
        )
        .updated_conversation;
        let result = handler().handle_blocking(IntentHandlerInput {
            conversation,
            analysis_intent: &IntentId::ReservationCreate,
            text: "10",
            analysis_entities: &[],
        });
        let workflow = result.updated_conversation.active_workflow().unwrap();
        assert_eq!(
            workflow.slot_value(SlotName::People),
            Some(&SlotDataValue::Number(10))
        );
        assert!(reply_text(&result.reply).contains("10 people"));
        assert!(reply_text(&result.reply).contains("19:00"));
        assert!(reply_text(&result.reply).contains("Do you confirm"));
    }

    #[test]
    fn number_word_entity_fills_people_slot() {
        let conversation = handle(
            Conversation::new(DomainType::Restaurant),
            IntentId::ReservationCreate,
            "",
            vec![
                entity("person", "Alice"),
                entity("date", "2099-06-12"),
                entity("time", "7pm"),
            ],
        )
        .updated_conversation;
        let result = handler().handle_blocking(IntentHandlerInput {
            conversation,
            analysis_intent: &IntentId::ReservationCreate,
            text: "six",
            analysis_entities: &[entity("people_count", "six")],
        });
        let workflow = result.updated_conversation.active_workflow().unwrap();
        assert_eq!(
            workflow.slot_value(SlotName::People),
            Some(&SlotDataValue::Number(6))
        );
        assert!(reply_text(&result.reply).contains("6 people"));
        assert!(reply_text(&result.reply).contains("19:00"));
        assert!(reply_text(&result.reply).contains("Do you confirm"));
    }

    #[test]
    fn negative_confirmation_keeps_workflow_open_for_changes() {
        let conversation = handle(
            Conversation::new(DomainType::Restaurant),
            IntentId::ReservationCreate,
            "",
            vec![
                entity("person", "Alice"),
                entity("date", "2099-06-12"),
                entity("time", "7pm"),
                entity("people_count", "4"),
            ],
        )
        .updated_conversation;
        let result = handle(conversation, IntentId::Negative, "", vec![]);
        assert_eq!(reply_text(&result.reply), "Okay. What would you like to change?");
        assert!(result.updated_conversation.has_active_workflow());
    }

    #[test]
    fn updated_slot_reasks_for_confirmation() {
        let conversation = handle(
            Conversation::new(DomainType::Restaurant),
            IntentId::ReservationCreate,
            "",
            vec![
                entity("person", "Alice"),
                entity("date", "2099-06-12"),
                entity("time", "7pm"),
                entity("people_count", "4"),
            ],
        )
        .updated_conversation;
        let result = handle(
            conversation,
            IntentId::Negative,
            "",
            vec![entity("people_count", "5")],
        );
        assert!(reply_text(&result.reply).contains("Do you confirm"));
        let workflow = result.updated_conversation.active_workflow().unwrap();
        assert_eq!(
            workflow.slot_value(SlotName::People),
            Some(&SlotDataValue::Number(5))
        );
    }

    #[test]
    fn affirmative_confirmation_completes_workflow() {
        let conversation = handle(
            Conversation::new(DomainType::Restaurant),
            IntentId::ReservationCreate,
            "",
            vec![
                entity("person", "Alice"),
                entity("date", "2099-06-12"),
                entity("time", "7pm"),
                entity("people_count", "4"),
            ],
        )
        .updated_conversation;
        let result = handle(conversation, IntentId::Affirmative, "", vec![]);
        assert!(reply_text(&result.reply).contains("REST-000001"));
        assert!(result.updated_conversation.is_idle());
        assert_eq!(
            result.updated_conversation.known_customer_name(),
            Some("Alice")
        );
        assert_eq!(
            result.updated_conversation.last_reservation_reference(),
            Some("REST-000001")
        );
    }

    #[test]
    fn past_date_triggers_constraint_error_and_re_prompts() {
        let conversation = Conversation::new(DomainType::Restaurant);
        let result = handle(
            conversation,
            IntentId::ReservationCreate,
            "",
            vec![entity("person", "Alice"), entity("date", "2000-01-01")],
        );
        assert_eq!(reply_text(&result.reply), "That date is in the past. Please provide a future date.");
        assert!(result.updated_conversation.has_active_workflow());
        assert!(
            result
                .updated_conversation
                .active_workflow()
                .unwrap()
                .slot_value(SlotName::Date)
                .is_none()
        );
    }

    #[test]
    fn people_out_of_range_triggers_constraint_error() {
        let conversation = Conversation::new(DomainType::Restaurant);
        let result = handle(
            conversation,
            IntentId::ReservationCreate,
            "",
            vec![
                entity("person", "Alice"),
                entity("date", "2099-06-12"),
                entity("time", "7pm"),
                entity("people_count", "50"),
            ],
        );
        assert!(!reply_text(&result.reply).contains("Do you confirm"));
        assert!(result.updated_conversation.has_active_workflow());
    }

    #[test]
    fn no_availability_clears_date_and_time_slots_and_suggests_next() {
        let conversation = handle(
            Conversation::new(DomainType::Restaurant),
            IntentId::ReservationCreate,
            "",
            vec![
                entity("person", "Alice"),
                entity("date", "2099-06-12"),
                entity("time", "7pm"),
                entity("people_count", "4"),
            ],
        )
        .updated_conversation;
        let reservation_service = build_service(RepositoryMode::NoAvailability);
        let result = ReservationCreateIntentHandler::new(reservation_service).handle_blocking(
            IntentHandlerInput {
                conversation,
                analysis_intent: &IntentId::Affirmative,
                text: "",
                analysis_entities: &[],
            },
        );
        assert!(reply_text(&result.reply).contains("21:00"));
        assert!(result.updated_conversation.has_active_workflow());
        let wf = result.updated_conversation.active_workflow().unwrap();
        assert!(wf.slot_value(SlotName::Date).is_none());
        assert!(wf.slot_value(SlotName::Time).is_none());
    }

    #[test]
    fn closed_restaurant_clears_date_and_time_and_reports_closed() {
        let conversation = handle(
            Conversation::new(DomainType::Restaurant),
            IntentId::ReservationCreate,
            "",
            vec![
                entity("person", "Alice"),
                entity("date", "2099-06-12"),
                entity("time", "7pm"),
                entity("people_count", "4"),
            ],
        )
        .updated_conversation;
        let reservation_service = build_service(RepositoryMode::Closed);
        let result = ReservationCreateIntentHandler::new(reservation_service).handle_blocking(
            IntentHandlerInput {
                conversation,
                analysis_intent: &IntentId::Affirmative,
                text: "",
                analysis_entities: &[],
            },
        );

        assert!(
            reply_text(&result.reply).contains("closed")
                || reply_text(&result.reply).contains("opening hours")
                || reply_text(&result.reply).contains("horaires")
        );
        assert!(result.updated_conversation.has_active_workflow());
        let wf = result.updated_conversation.active_workflow().unwrap();
        assert!(wf.slot_value(SlotName::Date).is_none());
        assert!(wf.slot_value(SlotName::Time).is_none());
    }
}
