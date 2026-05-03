package dev.stephyu.core.chat.application.state

import dev.stephyu.core.chat.domain.ConversationState

class ConversationStateMachine(
    private val idleStateHandler: IdleStateHandler,
    private val workflowStateHandler: WorkflowStateHandler,
) {
    fun process(input: ConversationStateInput): ConversationStateResult {
        val handler = when (input.session.state) {
            ConversationState.IDLE -> idleStateHandler
            ConversationState.WORKFLOW -> workflowStateHandler
        }
        return handler.process(input)
    }

}
