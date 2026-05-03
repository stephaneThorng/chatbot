package dev.stephyu.core.chat.adapter.`in`.web.mapper

import dev.stephyu.core.chat.adapter.`in`.web.dto.ChatMessageRequest
import dev.stephyu.core.chat.adapter.`in`.web.dto.ChatMessageResponse
import dev.stephyu.core.chat.application.command.ConversationResult
import dev.stephyu.core.chat.application.command.HandleConversationCommand

object ChatMessageWebMapper {
    fun toCommand(request: ChatMessageRequest): HandleConversationCommand =
        HandleConversationCommand(
            message = request.message,
            sessionId = request.sessionId,
        )

    fun toResponse(result: ConversationResult): ChatMessageResponse =
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


