package dev.stephyu.core.chat.application.usecase

import dev.stephyu.core.chat.application.command.ConversationResult
import dev.stephyu.core.chat.application.command.HandleConversationCommand
import dev.stephyu.core.chat.application.orchestration.ConversationOrchestrator
import dev.stephyu.core.chat.application.port.out.ConversationSessionRepository
import dev.stephyu.core.chat.domain.ConversationSession
import java.time.Clock
import java.time.Duration

class HandleConversationUseCase(
    private val conversationSessionRepository: ConversationSessionRepository,
    private val orchestrator: ConversationOrchestrator,
    private val clock: Clock,
    private val sessionTtl: Duration = Duration.ofMinutes(30),
) {
    suspend fun handle(command: HandleConversationCommand): ConversationResult {
        val now = clock.instant()
        val session = command.sessionId?.let { conversationSessionRepository.findActive(it, now) }
            ?: newSession(now)
        val orchestrationResult = orchestrator.handle(session, command.message.trim())
        val saved = save(orchestrationResult.session, now)

        return ConversationResult(
            sessionId = saved.id,
            reply = orchestrationResult.reply,
            intent = orchestrationResult.intent,
            conversationAct = orchestrationResult.conversationAct,
            state = saved.state,
            slots = orchestrationResult.slots,
            missingSlots = orchestrationResult.missingSlots,
            completed = orchestrationResult.completed,
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
