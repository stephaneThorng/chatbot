package dev.stephyu.core.chat.adapter.out.memory

import dev.stephyu.core.chat.application.port.out.ConversationSessionRepository
import dev.stephyu.core.chat.domain.ConversationSession
import java.time.Instant
import java.util.UUID
import java.util.concurrent.ConcurrentHashMap

class InMemoryConversationSessionRepository : ConversationSessionRepository {
    private val sessions = ConcurrentHashMap<String, ConversationSession>()

    override fun nextId(): String = UUID.randomUUID().toString()

    override fun findActive(sessionId: String, now: Instant): ConversationSession? {
        val session = sessions[sessionId] ?: return null
        if (session.expiresAt.isBefore(now)) {
            sessions.remove(sessionId)
            return null
        }
        return session
    }

    override fun save(session: ConversationSession) {
        sessions[session.id] = session
    }
}
