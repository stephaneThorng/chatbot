package dev.stephyu

import com.sun.net.httpserver.HttpServer
import dev.stephyu.core.chat.adapter.`in`.web.dto.ChatMessageResponse
import io.ktor.server.application.Application
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.client.statement.bodyAsText
import io.ktor.server.config.MapApplicationConfig
import io.ktor.http.ContentType
import io.ktor.http.HttpStatusCode
import io.ktor.http.contentType
import io.ktor.server.testing.testApplication
import java.net.InetSocketAddress
import kotlinx.serialization.json.Json
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class ServerTest {
    private val json = Json { ignoreUnknownKeys = true }

    @Test
    fun `chat endpoint creates a session`() = testApplication {
        application { installTestModules() }

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

    @Test
    fun `chat endpoint starts reservation workflow`() = testApplication {
        val mockNlp = mockNlpServer(
            """
            {
              "intent": {
                "name": "reservation_create",
                "confidence": 0.99,
                "source": "test",
                "alternatives": {}
              },
              "entities": []
            }
            """.trimIndent()
        )
        environment {
            config = MapApplicationConfig("nlp.apiUrl" to "http://localhost:${mockNlp.address.port}")
        }
        application { installTestModules() }

        try {
            val response = client.post("/api/v1/chat/messages") {
                contentType(ContentType.Application.Json)
                setBody("""{"message":"I want book a reservation"}""")
            }

            assertEquals(HttpStatusCode.OK, response.status)
            val body = json.decodeFromString<ChatMessageResponse>(response.bodyAsText())
            assertEquals("reservation_create", body.intent)
            assertEquals("WORKFLOW", body.state)
            assertTrue(body.reply.contains("name", ignoreCase = true))
        } finally {
            mockNlp.stop(0)
        }
    }

    private fun mockNlpServer(responseBody: String): HttpServer =
        HttpServer.create(InetSocketAddress(0), 0).apply {
            createContext("/analyze") { exchange ->
                val body = responseBody.toByteArray()
                exchange.responseHeaders.add("Content-Type", "application/json")
                exchange.sendResponseHeaders(200, body.size.toLong())
                exchange.responseBody.use { it.write(body) }
            }
            start()
        }

    private fun Application.installTestModules() {
        configureHttp()
        configureSerialization()
        configureKoin()
        configureRouting()
    }
}

