package dev.stephyu.core.chat.application.state

import dev.stephyu.core.chat.domain.ConversationSession
import dev.stephyu.core.chat.domain.SlotName

data class ChatStateTransition(
    val session: ConversationSession,
    val reply: String,
    val slots: Map<SlotName, String> = session.currentSlots(),
    val missingSlots: List<SlotName> = session.missingCurrentSlots(),
    val completed: Boolean = false,
)
