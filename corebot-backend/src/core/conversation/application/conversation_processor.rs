use std::collections::HashMap;
use std::sync::Arc;

use rust_i18n::t;

use super::intent_handler::{IntentHandlerInput, IntentHandlerRegistry, StateHandlerResult};
use super::intent_handlers::menu_item_details_handler::MenuItemDetailsIntentHandler;
use super::intent_handlers::opening_hours_handler::OpeningHoursIntentHandler;
use super::intent_handlers::reservation_cancel_handler::ReservationCancelIntentHandler;
use super::intent_handlers::reservation_create_handler::ReservationCreateIntentHandler;
use super::intent_handlers::static_reply_handler::StaticReplyIntentHandler;
use super::port::outbound::domain_gateway_trait::DomainGatewayPort;
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
    pub fn new(domain_gateway: Arc<dyn DomainGatewayPort>) -> Self {
        let restaurant_handlers: Vec<Arc<dyn super::intent_handler::IntentHandler>> = vec![
            Arc::new(ReservationCreateIntentHandler),
            Arc::new(ReservationCancelIntentHandler),
            Arc::new(OpeningHoursIntentHandler::new(domain_gateway)),
            Arc::new(MenuItemDetailsIntentHandler),
            Arc::new(StaticReplyIntentHandler::new(
                IntentId::Greeting,
                "intent.greeting.reply",
            )),
            Arc::new(StaticReplyIntentHandler::new(
                IntentId::Thanks,
                "intent.thanks.reply",
            )),
            Arc::new(StaticReplyIntentHandler::new(
                IntentId::Goodbye,
                "intent.goodbye.reply",
            )),
            Arc::new(StaticReplyIntentHandler::new(
                IntentId::Unknown("unknown".to_string()),
                "intent.unknown.reply",
            )),
        ];
        let mut intent_handlers_by_domain = HashMap::new();
        intent_handlers_by_domain.insert(
            DomainType::Restaurant,
            IntentHandlerRegistry::new(restaurant_handlers),
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
        conversation: &Conversation,
        message: &str,
        analysis: NluAnalysis,
    ) -> StateHandlerResult {
        // get intent_registry or return unknown response because of unmatching domain
        let Some(intent_registry) = self.handlers_for(conversation.domain) else {
            return StateHandlerResult {
                updated_conversation: conversation.clone(),
                reply: self.render_system_text(
                    "echo_intent",
                    &conversation.lang,
                    &[("intent".to_string(), analysis.intent.name)],
                ),
                handled_intent: IntentId::Unknown("unknown".to_string()),
            };
        };

        let analysis_intent = IntentId::from(&analysis.intent.name);
        let analysis_entities = analysis.entities;
        let analysis_policy = intent_registry.find_policy(&analysis_intent);

        if let Some(result) = self.process_workflow_turn(
            intent_registry,
            conversation,
            &analysis_intent,
            message,
            &analysis_entities,
            analysis_policy.as_ref(),
        ) {
            return result;
        }

        self.process_idle_intent(
            intent_registry,
            conversation,
            &conversation.lang,
            &analysis_intent,
            message,
            &analysis_entities,
        )
    }

    fn process_idle_intent(
        &self,
        intent_handlers: &IntentHandlerRegistry,
        conversation: &Conversation,
        lang: &str,
        intent: &IntentId,
        message: &str,
        entities: &[NluEntity],
    ) -> StateHandlerResult {
        if let Some(handler) = intent_handlers.get(intent) {
            return handler.handle(IntentHandlerInput {
                conversation,
                analysis_intent: intent,
                text: message,
                analysis_entities: entities,
            });
        }

        if intent == &IntentId::Cancel {
            return StateHandlerResult {
                updated_conversation: conversation.clone(),
                reply: self.render_system_text("no_active_workflow_to_cancel", lang, &[]),
                handled_intent: intent.clone(),
            };
        }

        StateHandlerResult {
            updated_conversation: conversation.clone(),
            reply: self.render_system_text(
                "echo_intent",
                lang,
                &[("intent".to_string(), intent.as_str().to_string())],
            ),
            handled_intent: intent.clone(),
        }
    }

    fn process_workflow_turn(
        &self,
        intent_handlers: &IntentHandlerRegistry,
        conversation: &Conversation,
        analysis_intent: &IntentId,
        message: &str,
        analysis_entities: &[NluEntity],
        analysis_policy: Option<&crate::core::conversation::domain::intent::IntentPolicy>,
    ) -> Option<StateHandlerResult> {
        let handler_intent =
            self.workflow_handler_intent(conversation, analysis_intent, analysis_policy)?;

        let handler = intent_handlers.get(&handler_intent)?;
        Some(handler.handle(IntentHandlerInput {
            conversation,
            analysis_intent,
            text: message,
            analysis_entities,
        }))
    }

    fn workflow_handler_intent(
        &self,
        conversation: &Conversation,
        analysis_intent: &IntentId,
        analysis_policy: Option<&crate::core::conversation::domain::intent::IntentPolicy>,
    ) -> Option<IntentId> {
        if let Some(workflow) = conversation.active_workflow() {
            return Some(workflow.intent.clone());
        }

        if analysis_policy.is_some_and(|policy| policy.kind == IntentKind::Workflow) {
            return Some(analysis_intent.clone());
        }

        None
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
    use crate::core::conversation::application::port::outbound::domain_gateway_trait::DomainGatewayPort;
    use crate::core::conversation::domain::model::domain_type::DomainType;
    use crate::core::conversation::domain::slot::EntityType;
    use crate::core::nlu_engine::domain::analysis::{
        NerTokenLabel, NluEntity, NluIntent, NluIntentCandidate,
    };

    struct StubDomainGateway;

    impl DomainGatewayPort for StubDomainGateway {
        fn get_opening_hours(&self) -> String {
            "Mon-Sun 9am-10pm".to_string()
        }
    }

    fn processor() -> ConversationProcessor {
        ConversationProcessor::new(Arc::new(StubDomainGateway))
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
            &conversation,
            "tell me about ramen",
            analysis(
                "ask_menu_item_details",
                vec![entity(EntityType::MenuItem, "ramen")],
            ),
        );

        assert_eq!(result.reply, "Here are the available details for ramen.");
        assert!(result.updated_conversation.is_idle());
        assert!(conversation.is_idle());
    }

    #[test]
    fn informational_intent_missing_ner_returns_custom_reply_and_stays_idle() {
        let conversation = Conversation::new(DomainType::Restaurant);

        let result = processor().process(
            &conversation,
            "tell me about a dish",
            analysis("ask_menu_item_details", vec![]),
        );

        assert_eq!(
            result.reply,
            "Which menu item or category would you like details about?"
        );
        assert!(result.updated_conversation.is_idle());
        assert!(conversation.is_idle());
    }

    #[test]
    fn workflow_intent_starts_workflow_and_prompts_next_slot() {
        let conversation = Conversation::new(DomainType::Restaurant);

        let result = processor().process(
            &conversation,
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
            &conversation,
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
            &conversation,
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
    fn active_workflow_ignores_informational_handler_routing() {
        let conversation = Conversation::new(DomainType::Restaurant);
        let p = processor();

        let start = p.process(
            &conversation,
            "book",
            analysis("reservation_create", vec![]),
        );
        let reply = p.process(
            &start.updated_conversation,
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
            &conversation,
            "surprise",
            analysis("not_in_catalog", vec![]),
        );

        assert_eq!(result.reply, "Detected intent: not_in_catalog");
        assert!(result.updated_conversation.is_idle());
        assert!(conversation.is_idle());
    }
}
