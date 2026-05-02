package dev.stephyu.core.chat.adapter.`in`.web.dto

import kotlinx.serialization.Serializable

@Serializable
data class ChatMessageResponse(
    val sessionId: String,
    val reply: String,
    val intent: String,
    val conversationAct: String?,
    val state: String,
    val slots: Map<String, String>,
    val missingSlots: List<String>,
    val completed: Boolean,
)
