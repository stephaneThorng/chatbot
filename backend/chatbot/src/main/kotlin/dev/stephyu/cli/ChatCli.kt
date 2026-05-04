package dev.stephyu.cli

import dev.stephyu.core.chat.adapter.`in`.web.dto.ChatMessageRequest
import dev.stephyu.core.chat.adapter.`in`.web.dto.ChatMessageResponse
import kotlinx.serialization.json.Json
import java.net.URI
import java.net.http.HttpClient
import java.net.http.HttpRequest
import java.net.http.HttpResponse

fun main(args: Array<String>) {
    val endpoint = args.firstOrNull()
        ?: System.getenv("CHATBOT_API_URL")
        ?: "http://localhost:8080/api/v1/chat/messages"

    TerminalChatClient(endpoint = endpoint).run()
}

private class TerminalChatClient(
    private val endpoint: String,
    private val httpClient: HttpClient = HttpClient.newHttpClient(),
    private val json: Json = Json { ignoreUnknownKeys = true },
) {
    private var sessionId: String? = null

    fun run() {
        println("Restaurant chatbot CLI")
        println("Endpoint: $endpoint")
        println("Type /exit to quit, /reset to start a new session.")
        println()

        while (true) {
            print("You: ")
            val input = readlnOrNull()?.trim() ?: break

            when {
                input.equals("/exit", ignoreCase = true) -> break
                input.equals("/quit", ignoreCase = true) -> break
                input.equals("/reset", ignoreCase = true) -> {
                    sessionId = null
                    println("Session reset.")
                }

                input.isBlank() -> println("Please type a message.")
                else -> send(input)
            }
        }
    }

    private fun send(message: String) {
        val payload = ChatMessageRequest(
            message = message,
            sessionId = sessionId,
        )
        val request = HttpRequest.newBuilder()
            .uri(URI.create(endpoint))
            .header("Content-Type", "application/json")
            .POST(HttpRequest.BodyPublishers.ofString(json.encodeToString(payload)))
            .build()

        val response = runCatching {
            httpClient.send(request, HttpResponse.BodyHandlers.ofString())
        }.getOrElse {
            println("Bot: I could not reach the backend at $endpoint.")
            return
        }

        if (response.statusCode() !in 200..299) {
            println("Bot: Backend returned HTTP ${response.statusCode()}: ${response.body()}")
            return
        }

        val body = runCatching {
            json.decodeFromString<ChatMessageResponse>(response.body())
        }.getOrElse {
            println("Bot: Backend returned an unreadable response: ${response.body()}")
            return
        }

        sessionId = body.sessionId
        println("Bot: ${body.reply}")
    }
}

