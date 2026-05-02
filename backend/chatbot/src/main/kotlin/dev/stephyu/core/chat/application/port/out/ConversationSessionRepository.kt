package dev.stephyu.core.chat.application.port.out

import dev.stephyu.core.chat.domain.ConversationSession
import java.time.Instant

interface ConversationSessionRepository {
    fun nextId(): String
    fun findActive(sessionId: String, now: Instant): ConversationSession?
    fun save(session: ConversationSession)
}
