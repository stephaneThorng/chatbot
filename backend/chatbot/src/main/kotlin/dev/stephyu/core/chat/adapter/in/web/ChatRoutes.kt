package dev.stephyu.core.chat.adapter.`in`.web

import dev.stephyu.core.chat.adapter.`in`.web.dto.ChatMessageRequest
import dev.stephyu.core.chat.adapter.`in`.web.mapper.ChatMessageWebMapper
import dev.stephyu.core.chat.application.usecase.HandleChatMessageUseCase
import io.ktor.http.HttpStatusCode
import io.ktor.server.application.call
import io.ktor.server.request.receive
import io.ktor.server.response.respond
import io.ktor.server.routing.Route
import io.ktor.server.routing.post
import io.ktor.server.routing.route
import org.koin.ktor.ext.getKoin

fun Route.chatRoutes() {
    route("/api/v1/chat") {
        post("/messages") {
            val useCase = call.application.getKoin().get<HandleChatMessageUseCase>()
            val request = call.receive<ChatMessageRequest>()
            val result = useCase.handle(ChatMessageWebMapper.toCommand(request))
            call.respond(HttpStatusCode.OK, ChatMessageWebMapper.toResponse(result))
        }
    }
}
