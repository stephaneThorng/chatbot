use crate::core::conversation::application::intent_handler::{
    IntentHandler, IntentHandlerInput, StateHandlerResult,
};
use crate::core::conversation::domain::model::intent::{IntentId, IntentKind, IntentPolicy};
use crate::core::conversation::domain::slot::EntityType;

pub struct MenuItemDetailsIntentHandler;

impl IntentHandler for MenuItemDetailsIntentHandler {
    fn intent(&self) -> IntentId {
        IntentId::AskMenuItemDetails
    }

    fn policy(&self) -> IntentPolicy {
        IntentPolicy {
            id: self.intent(),
            kind: IntentKind::Informational,
            nlu_task: None,
            workflow_slots: vec![],
            confirmation_prompt: None,
            completion_response: None,
        }
    }

    fn handle(&self, input: IntentHandlerInput<'_>) -> StateHandlerResult {
        let _ = (&input.conversation, input.analysis_intent, input.text);
        let menu_entity = self.lookup_entity_value(&input, EntityType::MenuItem);
        let allergen_entity = self.lookup_entity_value(&input, EntityType::Allergen);

        let reply = match (menu_entity, allergen_entity) {
            (Some(item), Some(allergen)) => {
                format!("I can check whether {item} contains {allergen}.")
            }
            (Some(item), None) => format!("Here are the available details for {item}."),
            (None, Some(allergen)) => format!("I can help find dishes that mention {allergen}."),
            (None, None) => "Which menu item or category would you like details about?".to_string(),
        };

        StateHandlerResult {
            updated_conversation: input.conversation,
            reply,
            handled_intent: self.intent(),
        }
    }
}
