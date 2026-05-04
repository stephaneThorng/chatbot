package dev.stephyu.core.chat.application.state

import dev.stephyu.core.chat.domain.intent.IntentName
import dev.stephyu.core.chat.domain.nlp.NlpAnalysis
import dev.stephyu.core.chat.domain.session.ConversationSession
import dev.stephyu.core.chat.domain.workflow.WorkflowCommand

/**
 * Normalized state input for a single conversation turn after preprocessing and intent resolution.
 */
data class StateHandlerInput(
    val session: ConversationSession,
    val intent: IntentName,
    val processedText: String,
    val analysis: NlpAnalysis,
    val workflowCommand: WorkflowCommand?,
    val processingMode: ProcessingMode = ProcessingMode.PRIMARY,
)


