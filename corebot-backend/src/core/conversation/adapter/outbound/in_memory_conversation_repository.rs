use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::core::conversation::application::port::outbound::conversation_repository_port::{
    ConversationRepositoryPort, RepositoryError,
};
use crate::core::conversation::domain::conversation::Conversation;
use crate::core::conversation::domain::conversation_id::ConversationId;

/// In-memory conversation storage for v1.
/// Thread-safe using RwLock.
pub struct InMemoryConversationRepository {
    store: Arc<RwLock<HashMap<ConversationId, Conversation>>>,
}

impl InMemoryConversationRepository {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryConversationRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl ConversationRepositoryPort for InMemoryConversationRepository {
    fn save(&self, conversation: &Conversation) -> Result<(), RepositoryError> {
        let mut store = self.store.write().map_err(|_| RepositoryError {
            message: "Failed to acquire write lock".to_string(),
        })?;

        store.insert(conversation.id, conversation.clone());
        Ok(())
    }

    fn load(&self, id: &ConversationId) -> Result<Option<Conversation>, RepositoryError> {
        let store = self.store.read().map_err(|_| RepositoryError {
            message: "Failed to acquire read lock".to_string(),
        })?;

        Ok(store.get(id).cloned())
    }

    fn delete(&self, id: &ConversationId) -> Result<(), RepositoryError> {
        let mut store = self.store.write().map_err(|_| RepositoryError {
            message: "Failed to acquire write lock".to_string(),
        })?;

        store.remove(id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::conversation::domain::domain_type::DomainType;

    #[test]
    fn save_and_load_conversation() {
        let repo = InMemoryConversationRepository::new();
        let conv = Conversation::new(DomainType::Restaurant);
        let conv_id = conv.id;

        repo.save(&conv).unwrap();
        let loaded = repo.load(&conv_id).unwrap();

        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().id, conv.id);
    }

    #[test]
    fn load_nonexistent_returns_none() {
        let repo = InMemoryConversationRepository::new();
        let loaded = repo.load(&ConversationId::new()).unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn delete_removes_conversation() {
        let repo = InMemoryConversationRepository::new();
        let conv = Conversation::new(DomainType::Restaurant);
        let conv_id = conv.id;

        repo.save(&conv).unwrap();
        repo.delete(&conv_id).unwrap();
        let loaded = repo.load(&conv_id).unwrap();

        assert!(loaded.is_none());
    }

    #[test]
    fn multiple_conversations() {
        let repo = InMemoryConversationRepository::new();
        let conv1 = Conversation::new(DomainType::Restaurant);
        let conv2 = Conversation::new(DomainType::Hotel);

        repo.save(&conv1).unwrap();
        repo.save(&conv2).unwrap();

        assert!(repo.load(&conv1.id).unwrap().is_some());
        assert!(repo.load(&conv2.id).unwrap().is_some());
    }
}
