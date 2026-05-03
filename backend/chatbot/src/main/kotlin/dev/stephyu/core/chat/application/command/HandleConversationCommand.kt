package dev.stephyu.core.chat.application.command

/**
 * Input command for the public chat message use case.
 */
data class HandleConversationCommand(
    val message: String,
    val sessionId: String?,
)


