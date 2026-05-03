package dev.stephyu.core.chat.application.state

import dev.stephyu.core.chat.domain.ConversationSession
import dev.stephyu.core.chat.domain.IntentName
import dev.stephyu.core.chat.domain.NlpAnalysis

data class ChatStateInput(
    val session: ConversationSession,
    val intent: IntentName,
    val message: String,
    val analysis: NlpAnalysis,
)
