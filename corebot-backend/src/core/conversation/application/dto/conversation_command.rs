#[derive(Debug)]
pub struct HandleConversationCommand {
    pub message: String,
    pub session_id: Option<String>,
}

#[derive(Debug)]
pub struct HandleConversationResult {
    pub session_id: String,
    pub reply: Vec<String>,
}
