package dev.stephyu.core.chat.application.command

import dev.stephyu.core.chat.domain.conversation.ConversationState
import dev.stephyu.core.chat.domain.conversation.ConversationAct
import dev.stephyu.core.chat.domain.intent.IntentName
import dev.stephyu.core.chat.domain.nlp.SlotName

/**
 * Public application result returned by the chat use case.
 */
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


