package dev.stephyu.core.chat.domain.reservation

import dev.stephyu.core.chat.domain.ConversationSession
import dev.stephyu.core.chat.domain.SlotName

data class ReservationTransition(
    val session: ConversationSession,
    val reply: String,
    val slots: Map<SlotName, String> = session.currentSlots(),
    val missingSlots: List<SlotName> = session.missingCurrentSlots(),
    val completed: Boolean = false,
)
