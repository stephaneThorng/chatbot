package dev.stephyu.core.chat.domain.workflow

import dev.stephyu.core.chat.domain.ConversationSession
import dev.stephyu.core.chat.domain.IntentName
import dev.stephyu.core.chat.domain.NlpAnalysis

data class WorkflowInput(
    val session: ConversationSession,
    val intent: IntentName,
    val message: String,
    val analysis: NlpAnalysis,
)
