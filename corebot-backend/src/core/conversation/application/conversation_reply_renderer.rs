use std::sync::Arc;

use rust_i18n::t;

use super::intent_handler::{IntentHandlerInput, IntentHandlerRegistry};
use super::intent_handlers::opening_hours_handler::OpeningHoursIntentHandler;
use super::port::outbound::domain_gateway_trait::DomainGatewayPort;
use crate::core::conversation::domain::intent::{IntentCatalog, IntentResponse};
use crate::core::conversation::domain::state_machine::ConversationEffect;

pub struct ConversationReplyRenderer {
    intent_handlers: IntentHandlerRegistry,
}

impl ConversationReplyRenderer {
    pub fn new(domain_gateway: Arc<dyn DomainGatewayPort>) -> Self {
        Self {
            intent_handlers: IntentHandlerRegistry::new(vec![Arc::new(
                OpeningHoursIntentHandler::new(domain_gateway),
            )]),
        }
    }

    pub fn render(
        &self,
        effect: &ConversationEffect,
        catalog: &IntentCatalog,
        lang: &str,
    ) -> String {
        match effect {
            ConversationEffect::IntentResponse {
                intent,
                text,
                entities,
            } => {
                if let Some(handler) = self.intent_handlers.get(intent) {
                    return handler
                        .handle(IntentHandlerInput {
                            intent,
                            text,
                            lang,
                            entities,
                        })
                        .reply;
                }

                let Some(policy) = catalog.get(intent) else {
                    return self.render_system_text(
                        catalog,
                        "echo_intent",
                        lang,
                        &[("intent".to_string(), intent.0.clone())],
                    );
                };

                match &policy.response {
                    IntentResponse::DomainOpeningHours => self.render_system_text(
                        catalog,
                        "echo_intent",
                        lang,
                        &[("intent".to_string(), intent.0.clone())],
                    ),
                    IntentResponse::Static(key) => self.translate_key(&key.0, lang),
                    IntentResponse::EchoIntent if intent.0 == "cancel" => {
                        self.render_system_text(catalog, "no_active_workflow_to_cancel", lang, &[])
                    }
                    IntentResponse::EchoIntent => self.render_system_text(
                        catalog,
                        "echo_intent",
                        lang,
                        &[("intent".to_string(), intent.0.clone())],
                    ),
                }
            }
            ConversationEffect::SystemText { key, params } => {
                self.render_system_text(catalog, key, lang, params)
            }
            ConversationEffect::SlotPrompt {
                workflow_intent,
                slot_name,
            } => catalog
                .slot_prompt_key(workflow_intent, slot_name)
                .map(|key| self.translate_key(key, lang))
                .unwrap_or_else(|| {
                    self.render_system_text(
                        catalog,
                        "missing_slot_fallback",
                        lang,
                        &[("slot".to_string(), slot_name.clone())],
                    )
                }),
            ConversationEffect::ConfirmationPrompt { workflow_intent } => catalog
                .confirmation_prompt_key(workflow_intent)
                .map(|key| self.translate_key(key, lang))
                .unwrap_or_else(|| self.render_system_text(catalog, "confirm_generic", lang, &[])),
            ConversationEffect::WorkflowCompletion { workflow_intent } => catalog
                .completion_response_key(workflow_intent)
                .map(|key| self.translate_key(key, lang))
                .unwrap_or_else(|| {
                    self.render_system_text(catalog, "workflow_complete", lang, &[])
                }),
        }
    }

    fn translate_key(&self, key: &str, lang: &str) -> String {
        t!(key, locale = lang).to_string()
    }

    fn render_system_text(
        &self,
        catalog: &IntentCatalog,
        system_key: &str,
        lang: &str,
        params: &[(String, String)],
    ) -> String {
        let Some(i18n_key) = catalog.system_text_key(system_key) else {
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
