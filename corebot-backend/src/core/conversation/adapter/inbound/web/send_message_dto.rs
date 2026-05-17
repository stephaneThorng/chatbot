use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub message: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SendMessageResponse {
    pub session_id: String,
    pub reply: Vec<String>,
}
