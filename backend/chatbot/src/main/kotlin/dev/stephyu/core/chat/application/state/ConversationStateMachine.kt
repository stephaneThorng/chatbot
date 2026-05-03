package dev.stephyu.core.chat.application.state

import dev.stephyu.core.chat.domain.ConversationState

class ChatStateMachine(
    private val idleStateHandler: IdleStateHandler,
    private val workflowStateHandler: WorkflowStateHandler,
) {
    fun process(input: ChatStateInput): ChatStateTransition =
        handlerFor(input.session.state).process(input)

    private fun handlerFor(state: ConversationState): ChatStateHandler =
        when (state) {
            ConversationState.IDLE -> idleStateHandler
            ConversationState.RESERVATION_CREATION -> workflowStateHandler
            ConversationState.RESERVATION_MODIFICATION -> workflowStateHandler
            ConversationState.RESERVATION_CANCELLATION -> workflowStateHandler
        }
}
