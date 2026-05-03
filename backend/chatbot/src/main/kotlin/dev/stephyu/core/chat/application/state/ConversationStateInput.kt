package dev.stephyu.core.chat.application.state

import dev.stephyu.core.chat.domain.ConversationSession
import dev.stephyu.core.chat.domain.IntentName
import dev.stephyu.core.chat.domain.NlpAnalysis
import dev.stephyu.core.chat.domain.workflow.WorkflowCommand

data class ConversationStateInput(
    val session: ConversationSession,
    val intent: IntentName,
    val message: String,
    val analysis: NlpAnalysis,
    val workflowCommand: WorkflowCommand? = null,
    val backgroundEnrichment: Boolean = false,
)
