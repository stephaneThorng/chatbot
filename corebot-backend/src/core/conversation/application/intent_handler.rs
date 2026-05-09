use std::collections::HashMap;
use std::sync::Arc;

use crate::core::conversation::domain::intent::IntentId;
use crate::core::conversation::domain::state_machine::DetectedEntity;

pub struct IntentHandlerInput<'a> {
    pub intent: &'a IntentId,
    pub text: &'a str,
    pub lang: &'a str,
    pub entities: &'a [DetectedEntity],
}

pub struct IntentHandlerResult {
    pub reply: String,
}

pub trait IntentHandler: Send + Sync {
    fn intent(&self) -> IntentId;
    fn handle(&self, input: IntentHandlerInput<'_>) -> IntentHandlerResult;
}

pub struct IntentHandlerRegistry {
    handlers: HashMap<IntentId, Arc<dyn IntentHandler>>,
}

impl IntentHandlerRegistry {
    pub fn new(handlers: Vec<Arc<dyn IntentHandler>>) -> Self {
        Self {
            handlers: handlers
                .into_iter()
                .map(|handler| (handler.intent(), handler))
                .collect(),
        }
    }

    pub fn get(&self, intent: &IntentId) -> Option<&dyn IntentHandler> {
        self.handlers.get(intent).map(Arc::as_ref)
    }
}
