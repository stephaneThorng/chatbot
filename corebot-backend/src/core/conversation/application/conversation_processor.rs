use std::collections::HashMap;

use rust_i18n::t;

use crate::core::conversation::application::dto::nlu_analysis_result::{
    NluAnalysisResult, NluEntityResult,
};
use crate::core::conversation::application::intent_handler::intent_handler::{
    IntentHandlerInput, IntentHandlerRegistry, StateHandlerResult,
};
use crate::core::conversation::domain::model::conversation::Conversation;
use crate::core::conversation::domain::model::domain_type::DomainType;
use crate::core::conversation::domain::model::intent::IntentId;

/// Application service that routes one decoded NLU turn to the right conversation path.
pub struct ConversationProcessor {
    intent_handlers: HashMap<DomainType, IntentHandlerRegistry>,
}

impl ConversationProcessor {
    pub fn new(
        restaurant_registry: IntentHandlerRegistry,
        hotel_registry: IntentHandlerRegistry,
    ) -> Self {
        let mut intent_handlers = HashMap::new();
        intent_handlers.insert(DomainType::Restaurant, restaurant_registry);
        intent_handlers.insert(DomainType::Hotel, hotel_registry);

        Self { intent_handlers }
    }

    fn handlers_for(&self, domain: DomainType) -> Option<&IntentHandlerRegistry> {
        self.intent_handlers.get(&domain)
    }

    pub fn process(
        &self,
        conversation: Conversation,
        message: &str,
        analysis: NluAnalysisResult,
    ) -> StateHandlerResult {
        let Some(intent_registry) = self.handlers_for(conversation.domain) else {
            let reply = self.render_system_text(
                "echo_intent",
                conversation.lang.as_str(),
                &[("intent".to_string(), analysis.intent_name)],
            );
            return StateHandlerResult {
                updated_conversation: conversation,
                reply,
                handled_intent: IntentId::Unknown("unknown".to_string()),
            };
        };

        let mut analysis_intent = IntentId::from(&analysis.intent_name);
        let analysis_config = intent_registry.find_config(&analysis_intent);
        if let Some(workflow) = conversation.active_workflow() {
            analysis_intent =
                self.resolve_active_workflow_intent(workflow, &analysis, &analysis_intent);
            let analysis_entities = analysis.entities;
            let handler = intent_registry
                .get(&workflow.intent)
                .expect("workflow handler must exist for active workflow");
            return handler.handle(IntentHandlerInput {
                conversation,
                analysis_intent: &analysis_intent,
                text: message,
                analysis_entities: &analysis_entities,
            });
        }

        if analysis_config.is_some_and(|cfg| cfg.workflow.is_workflow()) {
            let analysis_entities = analysis.entities;
            let handler = intent_registry
                .get(&analysis_intent)
                .expect("workflow handler must exist for workflow config");
            return handler.handle(IntentHandlerInput {
                conversation,
                analysis_intent: &analysis_intent,
                text: message,
                analysis_entities: &analysis_entities,
            });
        }

        self.process_idle_intent(
            intent_registry,
            conversation,
            analysis_intent,
            message,
            &analysis.entities,
        )
    }

    fn resolve_active_workflow_intent(
        &self,
        workflow: &crate::core::conversation::domain::model::workflow::Workflow,
        analysis: &NluAnalysisResult,
        analysis_intent: &IntentId,
    ) -> IntentId {
        if !workflow.is_ready_for_confirmation() || !analysis.entities.is_empty() {
            return analysis_intent.clone();
        }

        analysis
            .intent_candidates
            .iter()
            .filter_map(|candidate| {
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
            .unwrap_or_else(|| analysis_intent.clone())
    }

    fn process_idle_intent(
        &self,
        intent_handlers: &IntentHandlerRegistry,
        conversation: Conversation,
        intent: IntentId,
        message: &str,
        entities: &[NluEntityResult],
    ) -> StateHandlerResult {
        if let Some(handler) = intent_handlers.get(&intent) {
            return handler.handle(IntentHandlerInput {
                conversation,
                analysis_intent: &intent,
                text: message,
                analysis_entities: entities,
            });
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
    use std::sync::Arc;

    use super::*;
    use crate::core::conversation::application::dto::nlu_analysis_result::NluIntentCandidate;
    use crate::core::conversation::application::port::outbound::restaurant_information_port::RestaurantInformationPort;
    use crate::core::conversation::application::port::outbound::restaurant_queries::{
        EventQuery, FacilityQuery, LocationQuery, MenuDietaryQuery, MenuItemDetailsQuery,
        MenuQuery, PaymentMethodQuery, PriceQuery, ReservationCreateQuery, ReservationLookupQuery,
    };
    use crate::core::conversation::application::port::outbound::restaurant_reservation_port::RestaurantReservationPort;
    use crate::core::conversation::application::intent_handler::restaurant_handler_registry_factory::{
        RestaurantConversationDependencies, RestaurantHandlerRegistryFactory,
    };
    use crate::core::conversation::domain::model::domain_type::DomainType;

    struct StubInformationPort;

    impl RestaurantInformationPort for StubInformationPort {
        fn get_opening_hours(&self) -> String {
            "Mon-Sun 9am-10pm".to_string()
        }

        fn find_menu(&self, _: MenuQuery) -> String {
            "full_menu:".to_string()
        }

        fn find_menu_dietary(&self, _: MenuDietaryQuery) -> String {
            "dietary_no_filter:".to_string()
        }

        fn find_menu_item_details(&self, _: MenuItemDetailsQuery) -> String {
            "details_no_filter:".to_string()
        }

        fn find_location(&self, _: LocationQuery) -> String {
            "address:".to_string()
        }

        fn get_contact(&self) -> String {
            "contact:+33123456789|test@example.com".to_string()
        }

        fn find_payment_methods(&self, _: PaymentMethodQuery) -> String {
            "all_methods:cash".to_string()
        }

        fn find_price(&self, _: PriceQuery) -> String {
            "price_general:".to_string()
        }

        fn get_takeaway_info(&self) -> String {
            "takeaway:yes|Yes".to_string()
        }

        fn find_event_info(&self, _: EventQuery) -> String {
            "event_info:Yes".to_string()
        }

        fn find_facility_info(&self, _: FacilityQuery) -> String {
            "all_facilities:wifi".to_string()
        }

        fn get_accessibility_info(&self) -> String {
            "accessibility:yes|Yes".to_string()
        }

        fn get_entertainment_info(&self) -> String {
            "entertainment:yes|Live music".to_string()
        }
    }

    struct StubReservationPort;

    impl RestaurantReservationPort for StubReservationPort {
        fn create_reservation(&self, _: ReservationCreateQuery) -> Result<String, crate::core::conversation::application::port::outbound::restaurant_queries::ReservationFailure>{
            Ok("created:REST-NEW123".to_string())
        }

        fn check_reservation(&self, _: ReservationLookupQuery) -> String {
            "no_reference_or_name:".to_string()
        }
    }

    fn processor() -> ConversationProcessor {
        let restaurant_registry =
            RestaurantHandlerRegistryFactory::build(RestaurantConversationDependencies {
                information_port: Arc::new(StubInformationPort),
                reservation_port: Arc::new(StubReservationPort),
            });
        ConversationProcessor::new(restaurant_registry, IntentHandlerRegistry::new(vec![]))
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
        NluAnalysisResult {
            intent_name: intent_name.to_string(),
            intent_confidence: 1.0,
            intent_candidates,
            entities,
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

    #[test]
    fn informational_intent_with_ner_returns_handler_reply_and_stays_idle() {
        let conversation = Conversation::new(DomainType::Restaurant);

        let result = processor().process(
            conversation.clone(),
            "tell me about ramen",
            analysis("ask_menu_item_details", vec![entity("menu_item", "ramen")]),
        );

        assert_eq!(
            result.reply,
            "Which menu item would you like details about?"
        );
        assert!(result.updated_conversation.is_idle());
        assert!(conversation.is_idle());
    }

    #[test]
    fn informational_intent_missing_ner_returns_custom_reply_and_stays_idle() {
        let conversation = Conversation::new(DomainType::Restaurant);

        let result = processor().process(
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

        let result = processor().process(
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

        let result = processor().process(
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

        let result = processor().process(
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

        let result = processor().process(
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

        assert_eq!(result.reply, "Here is our pricing information: .");
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
            let result = processor().process(
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

        let start = processor.process(
            conversation.clone(),
            "book",
            analysis("reservation_create", vec![]),
        );
        let reply = processor.process(
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
        let started = processor.process(
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
                        confidence: 0.174,
                    },
                    NluIntentCandidate {
                        name: "negative".to_string(),
                        confidence: 0.073,
                    },
                ],
            ),
        );

        assert!(confirmed.reply.contains("REST-NEW123"));
        assert!(confirmed.updated_conversation.is_idle());
    }

    #[test]
    fn active_confirmation_prefers_negative_candidate_over_workflow_intent() {
        let processor = processor();
        let started = processor.process(
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
                        confidence: 0.210,
                    },
                ],
            ),
        );

        assert_eq!(rejected.reply, "Okay. What would you like to change?");
        assert!(rejected.updated_conversation.has_active_workflow());
    }

    #[test]
    fn unknown_intent_returns_deterministic_fallback() {
        let conversation = Conversation::new(DomainType::Restaurant);

        let result = processor().process(
            conversation.clone(),
            "surprise",
            analysis("not_in_catalog", vec![]),
        );

        assert_eq!(result.reply, "Detected intent: not_in_catalog");
        assert!(result.updated_conversation.is_idle());
        assert!(conversation.is_idle());
    }
}
