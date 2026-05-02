package dev.stephyu.core.chat.adapter.`in`.web.mapper

import dev.stephyu.core.chat.adapter.`in`.web.dto.ChatMessageRequest
import dev.stephyu.core.chat.adapter.`in`.web.dto.ChatMessageResponse
import dev.stephyu.core.chat.application.command.ChatMessageResult
import dev.stephyu.core.chat.application.command.HandleChatMessageCommand

object ChatMessageWebMapper {
    fun toCommand(request: ChatMessageRequest): HandleChatMessageCommand =
        HandleChatMessageCommand(
            message = request.message,
            sessionId = request.sessionId,
        )

    fun toResponse(result: ChatMessageResult): ChatMessageResponse =
        ChatMessageResponse(
            sessionId = result.sessionId,
            reply = result.reply,
            intent = result.intent.wireName,
            conversationAct = result.conversationAct?.wireName,
            state = result.state.name,
            slots = result.slots.mapKeys { it.key.wireName },
            missingSlots = result.missingSlots.map { it.wireName },
            completed = result.completed,
        )
}
