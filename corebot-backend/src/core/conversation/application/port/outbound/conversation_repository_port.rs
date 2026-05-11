use crate::core::conversation::domain::conversation::Conversation;
use crate::core::conversation::domain::conversation_id::ConversationId;

/// Outbound port - persistence interface for conversations.
/// Implementations are in adapter/outbound/.
pub trait ConversationRepositoryPort {
    /// Save or update a conversation.
    fn save(&self, conversation: &Conversation) -> Result<(), RepositoryError>;

    /// Load a conversation by ID.
    fn load(&self, id: &ConversationId) -> Result<Option<Conversation>, RepositoryError>;

    /// Delete a conversation by ID.
    fn delete(&self, id: &ConversationId) -> Result<(), RepositoryError>;
}

/// Repository errors.
#[derive(Debug, Clone, PartialEq)]
pub struct RepositoryError {
    pub message: String,
}
