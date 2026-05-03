package dev.stephyu.core.chat.application.usecase

import dev.stephyu.core.chat.application.command.ConversationResult
import dev.stephyu.core.chat.application.command.HandleConversationCommand
import dev.stephyu.core.chat.application.coordinator.ConversationCoordinator
import dev.stephyu.core.chat.application.port.out.ConversationSessionRepository
import dev.stephyu.core.chat.domain.session.ConversationSession
import java.time.Clock
import java.time.Duration

class HandleConversationUseCase(
    private val conversationSessionRepository: ConversationSessionRepository,
    private val coordinator: ConversationCoordinator,
    private val clock: Clock,
    private val sessionTtl: Duration = Duration.ofMinutes(30),
) {
    /**
     * Loads the current session, processes one message, and persists the refreshed in-memory session.
     */
    suspend fun handle(command: HandleConversationCommand): ConversationResult {
        val now = clock.instant()
        val session = command.sessionId?.let { conversationSessionRepository.findActive(it, now) }
            ?: newSession(now)
        val outcome = coordinator.handle(session, command.message.trim())
        val saved = save(outcome.session, now)

        return ConversationResult(
            sessionId = saved.id,
            reply = outcome.reply,
            intent = outcome.intent,
            conversationAct = outcome.conversationAct,
            state = saved.state,
            slots = outcome.slots,
            missingSlots = outcome.missingSlots,
            completed = outcome.completed,
        )
    }

    private fun newSession(now: java.time.Instant): ConversationSession =
        ConversationSession(
            id = conversationSessionRepository.nextId(),
            createdAt = now,
            updatedAt = now,
            expiresAt = now.plus(sessionTtl),
        )

    private fun save(session: ConversationSession, now: java.time.Instant): ConversationSession {
        val refreshed = session.withRefreshedTimestamps(now, now.plus(sessionTtl))
        conversationSessionRepository.save(refreshed)
        return refreshed
    }
}
