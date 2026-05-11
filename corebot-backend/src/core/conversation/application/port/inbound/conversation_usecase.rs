use crate::core::conversation::application::{HandleConversationCommand, HandleConversationResult};

/// Inbound port — defines what the web adapter can ask the application layer to do.
pub trait HandleConversationUseCase {
    fn handle_message(&self, command: HandleConversationCommand) -> HandleConversationResult;
}
