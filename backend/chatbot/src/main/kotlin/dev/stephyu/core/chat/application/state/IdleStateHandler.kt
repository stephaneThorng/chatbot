package dev.stephyu.core.chat.application.state

import dev.stephyu.core.chat.config.ConversationConfig
import dev.stephyu.core.chat.domain.ConversationState

class IdleStateHandler(
    private val conversationConfig: ConversationConfig,
) : StateHandler {
    override fun process(input: ConversationStateInput): ConversationStateResult =
        conversationConfig.findIntentService(input.intent)?.process(input)?.copy(
            handledIntent = input.intent,
        )
            ?: ConversationStateResult(
                session = input.session.copy(state = ConversationState.IDLE),
                reply = "I can help with reservations, opening hours, location, menu, prices, and contact details.",
            )
}
