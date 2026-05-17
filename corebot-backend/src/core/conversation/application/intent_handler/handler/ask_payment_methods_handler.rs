use rust_i18n::t;

use crate::core::conversation::application::intent_handler::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::application::port::outbound::restaurant::restaurant_business_info_repository_port::RestaurantBusinessInfoRepositoryPort;
use crate::core::conversation::domain::model::intent::{IntentConfig, IntentId, IntentWorkflow};

pub struct AskPaymentMethodsIntentHandler<'a, B> {
    business_info_repository: &'a B,
}

impl<'a, B> AskPaymentMethodsIntentHandler<'a, B> {
    pub fn new(business_info_repository: &'a B) -> Self {
        Self {
            business_info_repository,
        }
    }
}

#[async_trait::async_trait]
impl<B> IntentHandler for AskPaymentMethodsIntentHandler<'_, B>
where
    B: RestaurantBusinessInfoRepositoryPort + Send + Sync,
{
    fn intent(&self) -> IntentId {
        IntentId::AskPaymentMethods
    }

    fn config(&self) -> IntentConfig {
        IntentConfig {
            id: self.intent(),
            workflow: IntentWorkflow::Informational,
        }
    }

    async fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let lang = input.conversation.lang.as_str();
        let method = self.lookup_entity_value(&input, "payment_method");
        let reply = match self
            .business_info_repository
            .payment_methods(input.conversation.business_id)
            .await
        {
            Ok(methods) => {
                let all = methods
                    .iter()
                    .map(|payment_method| payment_method.method_code.clone())
                    .collect::<Vec<_>>()
                    .join(", ");
                if let Some(method) = method {
                    if methods.iter().any(|candidate| {
                        candidate
                            .method_code
                            .to_lowercase()
                            .contains(&method.to_lowercase())
                    }) {
                        t!(
                            "intent.ask_payment_methods.accepted.reply",
                            locale = lang,
                            method = method,
                            all = all
                        )
                        .to_string()
                    } else {
                        t!(
                            "intent.ask_payment_methods.not_accepted.reply",
                            locale = lang,
                            method = method,
                            all = all
                        )
                        .to_string()
                    }
                } else {
                    t!(
                        "intent.ask_payment_methods.all.reply",
                        locale = lang,
                        methods = all
                    )
                    .to_string()
                }
            }
            Err(_) => t!("intent.ask_payment_methods.reply", locale = lang).to_string(),
        };
        StateHandlerResult {
            updated_conversation: input.conversation,
            reply,
            handled_intent: self.intent(),
        }
    }
}
