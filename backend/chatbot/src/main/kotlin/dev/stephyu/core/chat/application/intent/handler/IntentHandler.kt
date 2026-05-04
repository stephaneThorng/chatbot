package dev.stephyu.core.chat.application.intent.handler

import dev.stephyu.core.chat.application.state.StateHandlerInput
import dev.stephyu.core.chat.application.state.StateHandlerResult
import dev.stephyu.core.chat.application.intent.policy.IntentPolicy
import dev.stephyu.core.chat.domain.session.ConversationSession
import dev.stephyu.core.chat.domain.intent.IntentName
import dev.stephyu.core.chat.domain.workflow.WorkflowDefinition

/**
 * Handles one business intent and optionally owns a workflow definition for that intent.
 */
interface IntentHandler {
    val intent: IntentName
    val policy: IntentPolicy
        get() = IntentPolicy()

    /**
     * Returns the workflow definition for the current session when the intent uses workflow progression.
     */
    fun workflowDefinition(session: ConversationSession): WorkflowDefinition? = null

    /**
     * Processes the current turn and returns the updated session plus reply payload.
     */
    fun process(input: StateHandlerInput): StateHandlerResult
}


