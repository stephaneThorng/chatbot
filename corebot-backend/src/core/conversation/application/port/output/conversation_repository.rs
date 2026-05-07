use crate::core::conversation::domain::conversation::Conversation;

/// Outbound port - persistence interface for conversations.
/// Implementations are in adapter/output/.
pub trait ConversationRepository: Send + Sync {
    /// Save or update a conversation.
    fn save(&self, conversation: &Conversation) -> Result<(), RepositoryError>;

    /// Load a conversation by ID.
    fn load(&self, id: &str) -> Result<Option<Conversation>, RepositoryError>;

    /// Delete a conversation by ID.
    fn delete(&self, id: &str) -> Result<(), RepositoryError>;
}

/// Repository errors.
#[derive(Debug, Clone, PartialEq)]
pub struct RepositoryError {
    pub message: String,
}