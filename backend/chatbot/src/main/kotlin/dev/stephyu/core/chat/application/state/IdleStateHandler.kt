package dev.stephyu.core.chat.application.state

import dev.stephyu.core.chat.application.intent.catalog.IntentCatalog
import dev.stephyu.core.chat.domain.conversation.ConversationState

class IdleStateHandler(
    private val intentCatalog: IntentCatalog,
) : StateHandler {
    override fun process(input: StateHandlerInput): StateHandlerResult =
        intentCatalog.findIntentHandler(input.intent)
            ?.process(input)
            ?.copy( handledIntentOverride = input.intent)
            ?: StateHandlerResult(
                updatedSession = input.session.copy(state = ConversationState.IDLE),
                reply = "I can help with reservations, opening hours, location, menu, prices, and contact details.",
            )
}


