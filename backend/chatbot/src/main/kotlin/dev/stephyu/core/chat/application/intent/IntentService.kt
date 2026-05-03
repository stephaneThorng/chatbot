package dev.stephyu.core.chat.application.intent

import dev.stephyu.core.chat.application.state.ConversationStateInput
import dev.stephyu.core.chat.application.state.ConversationStateResult
import dev.stephyu.core.chat.domain.ConversationSession
import dev.stephyu.core.chat.domain.IntentName
import dev.stephyu.core.chat.domain.workflow.WorkflowDefinition

interface IntentService {
    val intent: IntentName
    val policy: IntentPolicy
        get() = IntentPolicy()

    fun workflowDefinition(session: ConversationSession): WorkflowDefinition? = null

    fun process(input: ConversationStateInput): ConversationStateResult
}
