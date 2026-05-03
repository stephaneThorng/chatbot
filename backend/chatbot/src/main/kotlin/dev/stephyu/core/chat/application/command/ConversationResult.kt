package dev.stephyu.core.chat.application.command

import dev.stephyu.core.chat.domain.ConversationState
import dev.stephyu.core.chat.domain.ConversationAct
import dev.stephyu.core.chat.domain.IntentName
import dev.stephyu.core.chat.domain.SlotName

data class ConversationResult(
    val sessionId: String,
    val reply: String,
    val intent: IntentName,
    val conversationAct: ConversationAct?,
    val state: ConversationState,
    val slots: Map<SlotName, String>,
    val missingSlots: List<SlotName>,
    val completed: Boolean,
)
