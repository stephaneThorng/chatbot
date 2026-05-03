package dev.stephyu.core.chat.adapter.`in`.web.dto

import kotlinx.serialization.Serializable

@Serializable
data class ChatMessageRequest(
    val message: String,
    val sessionId: String? = null,
)


