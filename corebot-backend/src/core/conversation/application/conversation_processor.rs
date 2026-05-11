use std::collections::HashMap;
use std::sync::Arc;

use rust_i18n::t;

use super::intent_handler::{IntentHandlerInput, IntentHandlerRegistry, StateHandlerResult};
use super::port::outbound::domain_gateway_port::DomainGatewayPort;
use super::restaurant_handler_registry_factory::RestaurantHandlerRegistryFactory;
use crate::core::conversation::domain::date_resolver::DateResolver;
use crate::core::conversation::domain::model::conversation::Conversation;
use crate::core::conversation::domain::model::domain_type::DomainType;
use crate::core::conversation::domain::model::intent::{IntentId, IntentKind};
use crate::core::nlu_engine::domain::analysis::{NluAnalysis, NluEntity};

/// Application service that routes one decoded NLU turn to the right conversation path.
///
/// Workflow turns are delegated to the matching workflow handler. Idle
/// informational turns are delegated to stateless intent handlers and keep the
/// conversation idle.
pub struct ConversationProcessor {
    intent_handlers: HashMap<DomainType, IntentHandlerRegistry>,
}

impl ConversationProcessor {
    pub fn new<D: DomainGatewayPort + Send + Sync + 'static>(
        domain_gateway: D,
        date_resolver: Arc<dyn DateResolver>,
    ) -> Self {
        let gateway = Arc::new(domain_gateway);
        let mut intent_handlers_by_domain = HashMap::new();
        intent_handlers_by_domain.insert(
            DomainType::Restaurant,
            RestaurantHandlerRegistryFactory::build(Arc::clone(&gateway), date_resolver),
        );
        intent_handlers_by_domain.insert(DomainType::Hotel, IntentHandlerRegistry::new(vec![]));

        Self {
            intent_handlers: intent_handlers_by_domain,
        }
    }

    fn handlers_for(&self, domain: DomainType) -> Option<&IntentHandlerRegistry> {
        self.intent_handlers.get(&domain)
    }

    /// Processes one user turn after NLU inference.
    ///
    /// Active workflows and workflow-starting intents go through the FSM.
    /// Informational intents go through handlers or static catalog replies.
    pub fn process(
        &self,
        conversation: Conversation,
        message: &str,
        analysis: NluAnalysis,
    ) -> StateHandlerResult {
        // get intent_registry or return unknown response because of unmatching domain
        let Some(intent_registry) = self.handlers_for(conversation.domain) else {
            let reply = self.render_system_text(
                "echo_intent",
                conversation.lang.as_str(),
                &[("intent".to_string(), analysis.intent.name)],
            );
            return StateHandlerResult {
                updated_conversation: conversation,
                reply,
                handled_intent: IntentId::Unknown("unknown".to_string()),
            };
        };

        let analysis_intent = IntentId::from(&analysis.intent.name);
        let analysis_entities = analysis.entities;
        let analysis_policy = intent_registry.find_policy(&analysis_intent);
        if let Some(workflow) = conversation.active_workflow() {
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

        if analysis_policy.is_some_and(|policy| policy.kind == IntentKind::Workflow) {
            let handler = intent_registry
                .get(&analysis_intent)
                .expect("workflow handler must exist for workflow policy");
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
            &analysis_entities,
        )
    }

    fn process_idle_intent(
        &self,
        intent_handlers: &IntentHandlerRegistry,
        conversation: Conversation,
        intent: IntentId,
        message: &str,
        entities: &[NluEntity],
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
    use super::*;
    use crate::core::conversation::application::port::outbound::domain_gateway_port::DomainGatewayPort;
    use crate::core::conversation::domain::model::domain_type::DomainType;
    use crate::core::conversation::domain::slot::EntityType;
    use crate::core::nlu_engine::domain::analysis::{
        NerTokenLabel, NluEntity, NluIntent, NluIntentCandidate,
    };
    use std::sync::Arc;

    struct StubDomainGateway;

    impl DomainGatewayPort for StubDomainGateway {
        fn get_opening_hours(&self) -> String {
            "Mon-Sun 9am-10pm".to_string()
        }
        fn get_menu(&self, _: Option<&str>, _: Option<&str>, _: Option<&str>) -> String {
            "full_menu:".to_string()
        }
        fn get_menu_dietary(&self, _: Option<&str>) -> String {
            "dietary_no_filter:".to_string()
        }
        fn get_menu_item_details(&self, _: Option<&str>, _: Option<&str>) -> String {
            "details_no_filter:".to_string()
        }
        fn get_location(&self, _: Option<&str>) -> String {
            "address:".to_string()
        }
        fn get_contact(&self) -> String {
            "contact:+33123456789|test@example.com".to_string()
        }
        fn get_payment_methods(&self, _: Option<&str>) -> String {
            "all_methods:cash".to_string()
        }
        fn get_price(&self, _: Option<&str>, _: Option<&str>, _: Option<&str>) -> String {
            "price_general:".to_string()
        }
        fn get_takeaway_info(&self) -> String {
            "takeaway:yes|Yes".to_string()
        }
        fn get_event_info(&self, _: Option<&str>) -> String {
            "event_info:Yes".to_string()
        }
        fn get_facility_info(&self, _: Option<&str>) -> String {
            "all_facilities:wifi".to_string()
        }
        fn get_accessibility_info(&self) -> String {
            "accessibility:yes|Yes".to_string()
        }
        fn get_entertainment_info(&self) -> String {
            "entertainment:yes|Live music".to_string()
        }
        fn check_reservation(&self, _: Option<&str>) -> String {
            "no_reference:".to_string()
        }
    }

    fn processor() -> ConversationProcessor {
        use crate::core::conversation::domain::date_resolver::{DateResolveError, DateResolver};
        struct AlwaysOk;
        impl DateResolver for AlwaysOk {
            fn resolve(
                &self,
                _: &str,
                today: chrono::NaiveDate,
            ) -> Result<chrono::NaiveDate, DateResolveError> {
                Ok(today + chrono::Duration::days(1))
            }
        }
        ConversationProcessor::new(StubDomainGateway, Arc::new(AlwaysOk))
    }

    fn analysis(intent_name: &'static str, entities: Vec<NluEntity>) -> NluAnalysis {
        NluAnalysis {
            processed_text: String::new(),
            intent: NluIntent {
                name: intent_name.to_string(),
                confidence: 1.0,
            },
            intents: Vec::<NluIntentCandidate>::new(),
            entities,
            ner_labels: Vec::<NerTokenLabel>::new(),
        }
    }

    fn entity(entity_type: EntityType, value: &str) -> NluEntity {
        NluEntity {
            entity_type,
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
            analysis(
                "ask_menu_item_details",
                vec![entity(EntityType::MenuItem, "ramen")],
            ),
        );

        // Handler now calls domain gateway — stub returns "details_no_filter:" → fallback key
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

        assert_eq!(result.reply, "What name should I use for the reservation?");
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
                vec![entity(EntityType::ReservationReference, "ABC123")],
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
                    entity(EntityType::PriceComparator, "under"),
                    entity(EntityType::PriceAmount, "20 euros"),
                ],
            ),
        );

        // Stub returns "price_general:" → general.reply key
        assert_eq!(result.reply, "Here is our pricing information: .");
        assert!(result.updated_conversation.is_idle());
        assert!(conversation.is_idle());
    }

    #[test]
    fn restaurant_informational_intents_are_handled_without_echo_fallback() {
        // These intents must be routed to a dedicated handler (not echo fallback).
        // The stub gateway returns minimal payloads → each handler falls through to its
        // default i18n key. We only verify that the reply is NOT the echo fallback.
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
        let p = processor();

        let start = p.process(
            conversation.clone(),
            "book",
            analysis("reservation_create", vec![]),
        );
        let reply = p.process(
            start.updated_conversation,
            "what are your hours",
            analysis("ask_opening_hours", vec![]),
        );

        assert_eq!(reply.reply, "What name should I use for the reservation?");
        assert!(reply.updated_conversation.has_active_workflow());
        assert!(conversation.is_idle());
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
