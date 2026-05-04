package dev.stephyu.core.chat.application.state

import dev.stephyu.core.chat.domain.conversation.ConversationAct
import dev.stephyu.core.chat.domain.intent.IntentName
import dev.stephyu.core.chat.domain.nlp.SlotName
import dev.stephyu.core.chat.domain.session.ConversationSession

/**
 * Result produced by a state handler for a single turn.
 */
data class StateHandlerResult(
    val updatedSession: ConversationSession,
    val reply: String,
    val conversationAct: ConversationAct? = null,
    val completed: Boolean = false,
    val handledIntentOverride: IntentName? = null,
    val slotSnapshot: Map<SlotName, String>? = null,
    val missingSlotSnapshot: List<SlotName>? = null,
) {
    val handledIntent: IntentName?
        get() = handledIntentOverride ?: updatedSession.currentIntent

    val slots: Map<SlotName, String>
        get() = slotSnapshot ?: updatedSession.filledSlots()

    val missingSlots: List<SlotName>
        get() = missingSlotSnapshot ?: updatedSession.missingSlots()
}
