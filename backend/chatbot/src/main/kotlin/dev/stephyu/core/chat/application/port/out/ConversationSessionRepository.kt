package dev.stephyu.core.chat.application.port.out

import dev.stephyu.core.chat.domain.session.ConversationSession
import java.time.Instant

/**
 * Outbound repository port for transient chat sessions.
 */
interface ConversationSessionRepository {
    /**
     * Creates the next backend-owned session identifier.
     */
    fun nextId(): String

    /**
     * Returns the active session when it exists and has not expired.
     */
    fun findActive(sessionId: String, now: Instant): ConversationSession?

    /**
     * Persists the current in-memory session snapshot.
     */
    fun save(session: ConversationSession)
}


