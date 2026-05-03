package dev.stephyu.core.chat.application.state

import dev.stephyu.core.chat.domain.session.ConversationSession
import dev.stephyu.core.chat.domain.intent.IntentName
import dev.stephyu.core.chat.domain.nlp.SlotName

/**
 * Result produced by a state handler for a single turn.
 */
data class ConversationTurnResult(
    val session: ConversationSession,
    val reply: String,
    val slots: Map<SlotName, String> = session.filledSlots(),
    val missingSlots: List<SlotName> = session.missingSlots(),
    val completed: Boolean = false,
    val handledIntent: IntentName? = null,
)


