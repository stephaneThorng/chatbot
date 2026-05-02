package dev.stephyu.core.chat.application.command

data class HandleChatMessageCommand(
    val message: String,
    val sessionId: String?,
)
