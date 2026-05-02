package dev.stephyu

import dev.stephyu.core.chat.adapter.`in`.web.dto.ChatMessageResponse
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.client.statement.bodyAsText
import io.ktor.http.ContentType
import io.ktor.http.HttpStatusCode
import io.ktor.http.contentType
import io.ktor.server.testing.testApplication
import kotlinx.serialization.json.Json
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class ServerTest {
    private val json = Json { ignoreUnknownKeys = true }

    @Test
    fun `chat endpoint creates a session`() = testApplication {
        configure()

        val response = client.post("/api/v1/chat/messages") {
            contentType(ContentType.Application.Json)
            setBody("""{"message":"   "}""")
        }

        assertEquals(HttpStatusCode.OK, response.status)
        val body = json.decodeFromString<ChatMessageResponse>(response.bodyAsText())
        assertTrue(body.sessionId.isNotBlank())
        assertEquals("unknown", body.intent)
        assertEquals("IDLE", body.state)
    }
}
