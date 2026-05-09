use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;

use uuid::Uuid;

/// Strongly typed identifier for a conversation.
///
/// This value object prevents mixing conversation identifiers with unrelated
/// UUIDs and keeps repository contracts explicit.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConversationId(Uuid);

impl ConversationId {
    /// Creates a new random conversation identifier.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Wraps an existing UUID as a conversation identifier.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Returns the underlying UUID value.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for ConversationId {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for ConversationId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.0.fmt(f)
    }
}

impl From<Uuid> for ConversationId {
    fn from(value: Uuid) -> Self {
        Self::from_uuid(value)
    }
}

impl From<ConversationId> for Uuid {
    fn from(value: ConversationId) -> Self {
        value.0
    }
}

impl FromStr for ConversationId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(s).map(Self::from_uuid)
    }
}

#[cfg(test)]
mod tests {
    use super::ConversationId;
    use std::str::FromStr;

    #[test]
    fn round_trips_as_string() {
        let id = ConversationId::new();
        let parsed = ConversationId::from_str(&id.to_string()).unwrap();
        assert_eq!(parsed, id);
    }
}
