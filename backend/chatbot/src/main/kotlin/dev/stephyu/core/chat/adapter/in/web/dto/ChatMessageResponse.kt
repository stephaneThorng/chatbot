package dev.stephyu.core.chat.adapter.`in`.web.dto

import kotlinx.serialization.Serializable

@Serializable
data class ChatMessageResponse(
    val sessionId: String,
    val reply: String,
)


