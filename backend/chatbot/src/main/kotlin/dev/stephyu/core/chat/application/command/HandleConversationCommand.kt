package dev.stephyu.core.chat.application.command

data class HandleConversationCommand(
    val message: String,
    val sessionId: String?,
)
