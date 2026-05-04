package dev.stephyu.core.chat.application.state

import dev.stephyu.core.chat.domain.conversation.ConversationState

/**
 * Dispatches a turn to the appropriate coarse state handler.
 */
class ConversationStateDispatcher(
    private val idleStateHandler: IdleStateHandler,
    private val workflowStateHandler: WorkflowStateHandler,
) {
    /**
     * Routes the current turn to the handler selected by the session state.
     */
    fun process(input: StateHandlerInput): StateHandlerResult {
        val handler = when (input.session.state) {
            ConversationState.IDLE -> idleStateHandler
            ConversationState.WORKFLOW -> workflowStateHandler
        }
        return handler.process(input)
    }

}


