package dev.stephyu.core.chat.application.state

import dev.stephyu.core.chat.domain.ConversationSession
import dev.stephyu.core.chat.domain.SlotName

data class ConversationStateTransition(
    val session: ConversationSession,
    val reply: String,
    val slots: Map<SlotName, String> = session.filledSlots(),
    val missingSlots: List<SlotName> = session.missingSlots(),
    val completed: Boolean = false,
)
