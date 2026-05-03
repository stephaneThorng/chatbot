package dev.stephyu.core.chat.application.state

/**
 * Handles one coarse conversation state.
 */
interface StateHandler {
    fun process(input: ConversationTurnContext): ConversationTurnResult
}


