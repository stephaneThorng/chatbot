#[derive(Debug)]
pub struct HandleConversationCommand {
    pub message: String,
    pub session_id: Option<String>,
    pub lang: Option<String>,
    pub task: Option<String>,
}

#[derive(Debug)]
pub struct HandleConversationResult {
    pub session_id: String,
    pub reply: String,
    pub detected_intent: String,
}
