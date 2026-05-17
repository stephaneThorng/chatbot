use rust_i18n::t;

use crate::core::conversation::application::dto::nlu_analysis_result::{
    NluAnalysisResult, NluEntityResult,
};
use crate::core::conversation::application::intent_handler::intent_handler::{
    IntentHandlerInput, IntentHandlerRegistry, StateHandlerResult,
};
use crate::core::conversation::domain::model::conversation::Conversation;
use crate::core::conversation::domain::model::intent::IntentId;

const MIN_ACCEPTED_INTENT_CONFIDENCE: f32 = 0.3;

/// Application service that routes one decoded NLU turn to the right conversation path.
pub struct ConversationProcessor;

impl ConversationProcessor {
    pub fn new() -> Self {
        Self
    }

    #[cfg(test)]
    pub fn process(
        &self,
        intent_registry: &IntentHandlerRegistry<'_>,
        conversation: Conversation,
        message: &str,
        analysis: NluAnalysisResult,
    ) -> StateHandlerResult {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test runtime should be created")
            .block_on(self.process_async(intent_registry, conversation, message, analysis))
    }

    pub async fn process_async(
        &self,
        intent_registry: &IntentHandlerRegistry<'_>,
        conversation: Conversation,
        message: &str,
        analysis: NluAnalysisResult,
    ) -> StateHandlerResult {
        let mut analysis_intent = self.resolve_top_level_intent(&analysis);
        let analysis_config = intent_registry.find_config(&analysis_intent);
        if let Some(workflow) = conversation.active_workflow() {
            analysis_intent =
                self.resolve_active_workflow_intent(workflow, &analysis, analysis_intent);
            let analysis_entities = analysis.entities;
            let handler = intent_registry
                .get(&workflow.intent)
                .expect("workflow handler must exist for active workflow");
            return handler
                .handle(IntentHandlerInput {
                    conversation,
                    analysis_intent: &analysis_intent,
                    text: message,
                    analysis_entities: &analysis_entities,
                })
                .await;
        }

        if analysis_config.is_some_and(|cfg| cfg.workflow.is_workflow()) {
            let analysis_entities = analysis.entities;
            let handler = intent_registry
                .get(&analysis_intent)
                .expect("workflow handler must exist for workflow config");
            return handler
                .handle(IntentHandlerInput {
                    conversation,
                    analysis_intent: &analysis_intent,
                    text: message,
                    analysis_entities: &analysis_entities,
                })
                .await;
        }

        self.process_idle_intent(
            intent_registry,
            conversation,
            analysis_intent,
            message,
            &analysis.entities,
        )
        .await
    }

    fn resolve_active_workflow_intent(
        &self,
        workflow: &crate::core::conversation::domain::model::workflow::Workflow,
        analysis: &NluAnalysisResult,
        analysis_intent: IntentId,
    ) -> IntentId {
        if !workflow.is_ready_for_confirmation() || !analysis.entities.is_empty() {
            return analysis_intent;
        }

        analysis
            .intent_candidates
            .iter()
            .filter_map(|candidate| {
                if candidate.confidence < MIN_ACCEPTED_INTENT_CONFIDENCE {
                    return None;
                }
                let intent = IntentId::from(candidate.name.as_str());
                match intent {
                    IntentId::Affirmative | IntentId::Negative | IntentId::Cancel => {
                        Some((intent, candidate.confidence))
                    }
                    _ => None,
                }
            })
            .max_by(|left, right| left.1.total_cmp(&right.1))
            .map(|(intent, _)| intent)
            .unwrap_or(analysis_intent)
    }

    fn resolve_top_level_intent(&self, analysis: &NluAnalysisResult) -> IntentId {
        if analysis.intent_confidence < MIN_ACCEPTED_INTENT_CONFIDENCE {
            return IntentId::Unknown("unknown".to_string());
        }

        IntentId::from(&analysis.intent_name)
    }

    async fn process_idle_intent(
        &self,
        intent_handlers: &IntentHandlerRegistry<'_>,
        conversation: Conversation,
        intent: IntentId,
        message: &str,
        entities: &[NluEntityResult],
    ) -> StateHandlerResult {
        if let Some(handler) = intent_handlers.get(&intent) {
            return handler
                .handle(IntentHandlerInput {
                    conversation,
                    analysis_intent: &intent,
                    text: message,
                    analysis_entities: entities,
                })
                .await;
        }

        if intent == IntentId::Cancel {
            let reply = self.render_system_text(
                "no_active_workflow_to_cancel",
                conversation.lang.as_str(),
                &[],
            );
            return StateHandlerResult {
                updated_conversation: conversation,
                reply,
                handled_intent: intent,
            };
        }

        let reply = self.render_system_text(
            "echo_intent",
            conversation.lang.as_str(),
            &[("intent".to_string(), intent.as_str().to_string())],
        );
        StateHandlerResult {
            updated_conversation: conversation,
            reply,
            handled_intent: intent,
        }
    }

    fn translate_key(&self, key: &str, lang: &str) -> String {
        t!(key, locale = lang).to_string()
    }

    fn render_system_text(
        &self,
        system_key: &str,
        lang: &str,
        params: &[(String, String)],
    ) -> String {
        let Some(i18n_key) = system_text_i18n_key(system_key) else {
            return system_key.to_string();
        };
        let Some((arg_key, arg_value)) = params.first() else {
            return self.translate_key(i18n_key, lang);
        };

        match arg_key.as_str() {
            "intent" => t!(i18n_key, locale = lang, intent = arg_value.as_str()).to_string(),
            "slot" => t!(i18n_key, locale = lang, slot = arg_value.as_str()).to_string(),
            _ => self.translate_key(i18n_key, lang),
        }
    }
}

fn system_text_i18n_key(key: &str) -> Option<&'static str> {
    match key {
        "no_active_workflow" => Some("system.no_active_workflow"),
        "no_active_workflow_to_cancel" => Some("system.no_active_workflow_to_cancel"),
        "workflow_cancelled" => Some("system.workflow_cancelled"),
        "confirm_yes_no" => Some("system.confirm_yes_no"),
        "workflow_complete" => Some("system.workflow_complete"),
        "echo_intent" => Some("system.echo_intent"),
        "missing_slot_fallback" => Some("system.missing_slot_fallback"),
        "confirm_generic" => Some("system.confirm_generic"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::core::conversation::application::dto::nlu_analysis_result::NluIntentCandidate;
    use crate::core::conversation::application::port::outbound::restaurant::restaurant_availability_repository_port::RestaurantAvailabilityRepositoryPort;
    use crate::core::conversation::application::port::outbound::restaurant::restaurant_business_info_repository_port::RestaurantBusinessInfoRepositoryPort;
    use crate::core::conversation::application::port::outbound::restaurant::restaurant_menu_repository_port::RestaurantMenuRepositoryPort;
    use crate::core::conversation::application::port::outbound::restaurant::restaurant_reservation_repository_port::RestaurantReservationRepositoryPort;
    use crate::core::conversation::application::intent_handler::restaurant_handler_registry_factory::{
        RestaurantConversationDependencies, RestaurantHandlerRegistryFactory,
    };
    use crate::core::conversation::application::service::restaurant::{
        ConversationRestaurantMenuService, ConversationRestaurantReservationService,
    };
    use crate::core::conversation::domain::model::domain_type::DomainType;
    use crate::core::conversation::domain::restaurant::model::{
        BusinessFact, BusinessLocation, ContactChannel, EventSpace, Facility, MenuItem,
        MenuPriceFilter, OpeningHours, PaymentMethod, Reservation, ReservationDraft,
        ReservationSettings, RestaurantRepositoryError, TableType,
    };
    use chrono::{NaiveDate, NaiveTime, Weekday};
    use uuid::Uuid;

    #[derive(Clone)]
    struct StubRestaurantRepository {
        reservations: Arc<Mutex<Vec<Reservation>>>,
    }

    impl StubRestaurantRepository {
        fn new() -> Self {
            Self {
                reservations: Arc::new(Mutex::new(vec![])),
            }
        }
    }

    fn opening_hours() -> Vec<OpeningHours> {
        [
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
            Weekday::Sat,
            Weekday::Sun,
        ]
        .into_iter()
        .map(|day| OpeningHours {
            day_of_week: day,
            opens_at: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            closes_at: NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            is_closed: false,
        })
        .collect()
    }

    #[async_trait::async_trait]
    impl RestaurantBusinessInfoRepositoryPort for StubRestaurantRepository {
        async fn opening_hours(
            &self,
            _: Uuid,
        ) -> Result<Vec<OpeningHours>, RestaurantRepositoryError> {
            Ok(opening_hours())
        }

        async fn location(
            &self,
            _: Uuid,
        ) -> Result<Option<BusinessLocation>, RestaurantRepositoryError> {
            Ok(Some(BusinessLocation {
                address_line: "12 Rue de la Paix".to_string(),
                nearby_description: Some("near Central Station".to_string()),
            }))
        }

        async fn contact_channels(
            &self,
            _: Uuid,
        ) -> Result<Vec<ContactChannel>, RestaurantRepositoryError> {
            Ok(vec![
                ContactChannel {
                    channel_type: "phone".to_string(),
                    value: "+33123456789".to_string(),
                },
                ContactChannel {
                    channel_type: "email".to_string(),
                    value: "test@example.com".to_string(),
                },
            ])
        }

        async fn payment_methods(
            &self,
            _: Uuid,
        ) -> Result<Vec<PaymentMethod>, RestaurantRepositoryError> {
            Ok(vec![PaymentMethod {
                method_code: "cash".to_string(),
            }])
        }

        async fn facilities(&self, _: Uuid) -> Result<Vec<Facility>, RestaurantRepositoryError> {
            Ok(vec![Facility {
                facility_code: "wifi".to_string(),
                label: "wifi".to_string(),
            }])
        }

        async fn facts(
            &self,
            _: Uuid,
            _: &str,
        ) -> Result<Vec<BusinessFact>, RestaurantRepositoryError> {
            Ok(vec![
                BusinessFact {
                    fact_type: "takeaway".to_string(),
                    title: None,
                    content: "Yes".to_string(),
                },
                BusinessFact {
                    fact_type: "accessibility".to_string(),
                    title: None,
                    content: "Yes".to_string(),
                },
                BusinessFact {
                    fact_type: "entertainment".to_string(),
                    title: None,
                    content: "Live music".to_string(),
                },
            ])
        }

        async fn event_spaces(
            &self,
            _: Uuid,
        ) -> Result<Vec<EventSpace>, RestaurantRepositoryError> {
            Ok(vec![EventSpace {
                name: "main room".to_string(),
                description: Some("Yes".to_string()),
                contact: None,
            }])
        }
    }

    #[async_trait::async_trait]
    impl RestaurantMenuRepositoryPort for StubRestaurantRepository {
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
            _: &MenuPriceFilter,
        ) -> Result<Vec<MenuItem>, RestaurantRepositoryError> {
            Ok(vec![])
        }
    }

    #[async_trait::async_trait]
    impl RestaurantReservationRepositoryPort for StubRestaurantRepository {
        async fn next_reference_index(&self, _: Uuid) -> Result<i64, RestaurantRepositoryError> {
            Ok(self.reservations.lock().unwrap().len() as i64 + 1)
        }

        async fn create_reservation(
            &self,
            _: Uuid,
            reservation: ReservationDraft,
        ) -> Result<Reservation, RestaurantRepositoryError> {
            let reservation = Reservation {
                reference: reservation.reference,
                name: reservation.name,
                date: reservation.date,
                time: reservation.time,
                people_count: reservation.people_count,
            };
            self.reservations.lock().unwrap().push(reservation.clone());
            Ok(reservation)
        }

        async fn find_by_reference(
            &self,
            _: Uuid,
            reference: &str,
        ) -> Result<Option<Reservation>, RestaurantRepositoryError> {
            Ok(self
                .reservations
                .lock()
                .unwrap()
                .iter()
                .find(|reservation| reservation.reference.eq_ignore_ascii_case(reference))
                .cloned())
        }

        async fn find_by_name(
            &self,
            _: Uuid,
            name: &str,
        ) -> Result<Vec<Reservation>, RestaurantRepositoryError> {
            Ok(self
                .reservations
                .lock()
                .unwrap()
                .iter()
                .filter(|reservation| reservation.name.eq_ignore_ascii_case(name))
                .cloned()
                .collect())
        }

        async fn cancel_by_reference(
            &self,
            _: Uuid,
            reference: &str,
        ) -> Result<Option<Reservation>, RestaurantRepositoryError> {
            let mut reservations = self.reservations.lock().unwrap();
            let Some(index) = reservations
                .iter()
                .position(|reservation| reservation.reference.eq_ignore_ascii_case(reference))
            else {
                return Ok(None);
            };
            Ok(Some(reservations.remove(index)))
        }
    }

    #[async_trait::async_trait]
    impl RestaurantAvailabilityRepositoryPort for StubRestaurantRepository {
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
                capacity: 6,
                count: 2,
            }])
        }

        async fn opening_hours(
            &self,
            _: Uuid,
        ) -> Result<Vec<OpeningHours>, RestaurantRepositoryError> {
            Ok(opening_hours())
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
            Ok(self.reservations.lock().unwrap().clone())
        }
    }

    fn processor() -> ConversationProcessor {
        ConversationProcessor::new()
    }

    fn analysis(intent_name: &'static str, entities: Vec<NluEntityResult>) -> NluAnalysisResult {
        NluAnalysisResult {
            intent_name: intent_name.to_string(),
            intent_confidence: 1.0,
            intent_candidates: Vec::<NluIntentCandidate>::new(),
            entities,
        }
    }

    fn analysis_with_candidates(
        intent_name: &'static str,
        entities: Vec<NluEntityResult>,
        intent_candidates: Vec<NluIntentCandidate>,
    ) -> NluAnalysisResult {
        analysis_with_candidates_and_confidence(intent_name, 1.0, entities, intent_candidates)
    }

    fn analysis_with_candidates_and_confidence(
        intent_name: &'static str,
        intent_confidence: f32,
        entities: Vec<NluEntityResult>,
        intent_candidates: Vec<NluIntentCandidate>,
    ) -> NluAnalysisResult {
        NluAnalysisResult {
            intent_name: intent_name.to_string(),
            intent_confidence,
            intent_candidates,
            entities,
        }
    }

    fn analysis_with_confidence(
        intent_name: &'static str,
        intent_confidence: f32,
        entities: Vec<NluEntityResult>,
    ) -> NluAnalysisResult {
        NluAnalysisResult {
            intent_name: intent_name.to_string(),
            intent_confidence,
            intent_candidates: Vec::<NluIntentCandidate>::new(),
            entities,
        }
    }

    fn registry() -> IntentHandlerRegistry<'static> {
        let repository = StubRestaurantRepository::new();
        let business_info_repository = repository.clone();
        let menu_repository = repository.clone();
        let reservation_repository = repository.clone();
        let availability_repository = repository;
        let business_info_repository = Box::leak(Box::new(business_info_repository));
        let menu_service = Box::leak(Box::new(ConversationRestaurantMenuService::new(
            menu_repository,
        )));
        let reservation_service =
            Box::leak(Box::new(ConversationRestaurantReservationService::new(
                reservation_repository,
                availability_repository,
            )));
        RestaurantHandlerRegistryFactory::build(RestaurantConversationDependencies {
            business_info_repository,
            menu_service,
            reservation_service,
        })
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

    #[test]
    fn informational_intent_with_ner_returns_handler_reply_and_stays_idle() {
        let conversation = Conversation::new(DomainType::Restaurant);

        let registry = registry();
        let result = processor().process(
            &registry,
            conversation.clone(),
            "tell me about ramen",
            analysis("ask_menu_item_details", vec![entity("menu_item", "ramen")]),
        );

        assert_eq!(result.reply, "I couldn't find that item on our menu.");
        assert!(result.updated_conversation.is_idle());
        assert!(conversation.is_idle());
    }

    #[test]
    fn informational_intent_missing_ner_returns_custom_reply_and_stays_idle() {
        let conversation = Conversation::new(DomainType::Restaurant);

        let registry = registry();
        let result = processor().process(
            &registry,
            conversation.clone(),
            "tell me about a dish",
            analysis("ask_menu_item_details", vec![]),
        );

        assert_eq!(
            result.reply,
            "Which menu item would you like details about?"
        );
        assert!(result.updated_conversation.is_idle());
        assert!(conversation.is_idle());
    }

    #[test]
    fn workflow_intent_starts_workflow_and_prompts_next_slot() {
        let conversation = Conversation::new(DomainType::Restaurant);

        let registry = registry();
        let result = processor().process(
            &registry,
            conversation.clone(),
            "book",
            analysis("reservation_create", vec![]),
        );

        assert!(
            result
                .reply
                .ends_with("What name should I use for the reservation?")
        );
        assert!(result.updated_conversation.has_active_workflow());
        assert!(conversation.is_idle());
    }

    #[test]
    fn reservation_cancel_starts_workflow_and_prompts_reference() {
        let conversation = Conversation::new(DomainType::Restaurant);

        let registry = registry();
        let result = processor().process(
            &registry,
            conversation.clone(),
            "cancel my booking",
            analysis("reservation_cancel", vec![]),
        );

        assert_eq!(result.reply, "What is the reservation reference?");
        assert!(result.updated_conversation.has_active_workflow());
        assert!(conversation.is_idle());
    }

    #[test]
    fn reservation_cancel_with_reference_asks_for_confirmation() {
        let conversation = Conversation::new(DomainType::Restaurant);

        let registry = registry();
        let result = processor().process(
            &registry,
            conversation.clone(),
            "cancel ABC123",
            analysis(
                "reservation_cancel",
                vec![entity("reservation_reference", "ABC123")],
            ),
        );

        assert_eq!(
            result.reply,
            "I have the cancellation details. Do you confirm the cancellation?"
        );
        assert!(result.updated_conversation.has_active_workflow());
        assert!(conversation.is_idle());
    }

    #[test]
    fn price_condition_returns_deterministic_reply() {
        let conversation = Conversation::new(DomainType::Restaurant);

        let registry = registry();
        let result = processor().process(
            &registry,
            conversation.clone(),
            "do you have meals under 20 euros",
            analysis(
                "ask_price",
                vec![
                    entity("price_comparator", "under"),
                    entity("price_amount", "20 euros"),
                ],
            ),
        );

        assert_eq!(
            result.reply,
            "I can help with restaurant pricing information."
        );
        assert!(result.updated_conversation.is_idle());
        assert!(conversation.is_idle());
    }

    #[test]
    fn restaurant_informational_intents_are_handled_without_echo_fallback() {
        let informational_intents = [
            "ask_menu_general",
            "ask_menu_dietary",
            "ask_location",
            "ask_contact",
            "ask_payment_methods",
            "ask_price",
            "ask_takeaway_delivery",
            "ask_event",
            "ask_facilities",
            "ask_accessibility",
            "ask_entertainment",
            "check_reservation",
        ];

        for intent in informational_intents {
            let registry = registry();
            let result = processor().process(
                &registry,
                Conversation::new(DomainType::Restaurant),
                "",
                analysis(intent, vec![]),
            );

            assert!(
                !result.reply.starts_with("Detected intent:"),
                "intent {intent} should be handled, not echoed; got: {}",
                result.reply
            );
            assert!(result.updated_conversation.is_idle());
        }
    }

    #[test]
    fn active_workflow_ignores_informational_handler_routing() {
        let conversation = Conversation::new(DomainType::Restaurant);
        let processor = processor();
        let registry = registry();

        let start = processor.process(
            &registry,
            conversation.clone(),
            "book",
            analysis("reservation_create", vec![]),
        );
        let reply = processor.process(
            &registry,
            start.updated_conversation,
            "what are your hours",
            analysis("ask_opening_hours", vec![]),
        );

        assert_eq!(reply.reply, "What name should I use for the reservation?");
        assert!(reply.updated_conversation.has_active_workflow());
        assert!(conversation.is_idle());
    }

    #[test]
    fn active_confirmation_prefers_affirmative_candidate_over_workflow_intent() {
        let processor = processor();
        let registry = registry();
        let started = processor.process(
            &registry,
            Conversation::new(DomainType::Restaurant),
            "book",
            analysis(
                "reservation_create",
                vec![
                    entity("person", "Alice"),
                    entity("date", "2099-06-12"),
                    entity("time", "7pm"),
                    entity("people_count", "4"),
                ],
            ),
        );

        let confirmed = processor.process(
            &registry,
            started.updated_conversation,
            "yes",
            analysis_with_candidates(
                "reservation_create",
                vec![],
                vec![
                    NluIntentCandidate {
                        name: "reservation_create".to_string(),
                        confidence: 0.305,
                    },
                    NluIntentCandidate {
                        name: "affirmative".to_string(),
                        confidence: 0.841,
                    },
                    NluIntentCandidate {
                        name: "negative".to_string(),
                        confidence: 0.073,
                    },
                ],
            ),
        );

        assert!(confirmed.reply.contains("REST-000001"));
        assert!(confirmed.updated_conversation.is_idle());
    }

    #[test]
    fn active_confirmation_prefers_negative_candidate_over_workflow_intent() {
        let processor = processor();
        let registry = registry();
        let started = processor.process(
            &registry,
            Conversation::new(DomainType::Restaurant),
            "book",
            analysis(
                "reservation_create",
                vec![
                    entity("person", "Alice"),
                    entity("date", "2099-06-12"),
                    entity("time", "7pm"),
                    entity("people_count", "4"),
                ],
            ),
        );

        let rejected = processor.process(
            &registry,
            started.updated_conversation,
            "no",
            analysis_with_candidates(
                "reservation_create",
                vec![],
                vec![
                    NluIntentCandidate {
                        name: "reservation_create".to_string(),
                        confidence: 0.320,
                    },
                    NluIntentCandidate {
                        name: "negative".to_string(),
                        confidence: 0.812,
                    },
                ],
            ),
        );

        assert_eq!(rejected.reply, "Okay. What would you like to change?");
        assert!(rejected.updated_conversation.has_active_workflow());
    }

    #[test]
    fn active_confirmation_does_not_accept_choice_candidates_below_threshold() {
        let processor = processor();
        let registry = registry();
        let started = processor.process(
            &registry,
            Conversation::new(DomainType::Restaurant),
            "book",
            analysis(
                "reservation_create",
                vec![
                    entity("person", "Alice"),
                    entity("date", "2099-06-12"),
                    entity("time", "7pm"),
                    entity("people_count", "4"),
                ],
            ),
        );

        let pending = processor.process(
            &registry,
            started.updated_conversation,
            "no",
            analysis_with_candidates_and_confidence(
                "affirmative",
                0.159,
                vec![],
                vec![
                    NluIntentCandidate {
                        name: "affirmative".to_string(),
                        confidence: 0.159,
                    },
                    NluIntentCandidate {
                        name: "negative".to_string(),
                        confidence: 0.134,
                    },
                ],
            ),
        );

        assert_eq!(
            pending.reply,
            "I have the reservation details: Alice, Friday June 12 2099 at 19:00, for 4 people. Do you confirm this reservation?"
        );
        assert!(pending.updated_conversation.has_active_workflow());
    }

    #[test]
    fn unknown_intent_returns_deterministic_fallback() {
        let conversation = Conversation::new(DomainType::Restaurant);

        let registry = registry();
        let result = processor().process(
            &registry,
            conversation.clone(),
            "surprise",
            analysis("not_in_catalog", vec![]),
        );

        assert_eq!(result.reply, "Detected intent: not_in_catalog");
        assert!(result.updated_conversation.is_idle());
        assert!(conversation.is_idle());
    }

    #[test]
    fn low_confidence_known_intent_falls_back_to_unknown_handler() {
        let conversation = Conversation::new(DomainType::Restaurant);

        let registry = registry();
        let result = processor().process(
            &registry,
            conversation.clone(),
            "que pense tu des arc en rust ?",
            analysis_with_confidence("ask_menu_dietary", 0.199, vec![]),
        );

        assert_eq!(result.reply, "I did not understand that request.");
        assert_eq!(
            result.handled_intent,
            IntentId::Unknown("unknown".to_string())
        );
        assert!(result.updated_conversation.is_idle());
        assert!(conversation.is_idle());
    }
}
