package dev.stephyu.core.chat.application.usecase

import dev.stephyu.core.chat.application.command.ChatMessageResult
import dev.stephyu.core.chat.application.command.HandleChatMessageCommand
import dev.stephyu.core.chat.application.orchestration.ChatMessageOrchestrator
import dev.stephyu.core.chat.application.port.out.ConversationSessionRepository
import dev.stephyu.core.chat.domain.ConversationSession
import java.time.Clock
import java.time.Duration

class HandleChatMessageUseCase(
    private val sessions: ConversationSessionRepository,
    private val orchestrator: ChatMessageOrchestrator,
    private val clock: Clock,
    private val sessionTtl: Duration = Duration.ofMinutes(30),
) {
    suspend fun handle(command: HandleChatMessageCommand): ChatMessageResult {
        val now = clock.instant()
        val session = command.sessionId?.let { sessions.findActive(it, now) } ?: newSession(now)
        val orchestrationResult = orchestrator.handle(session, command.message.trim())
        val saved = save(orchestrationResult.session, now)

        return ChatMessageResult(
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
            id = sessions.nextId(),
            createdAt = now,
            updatedAt = now,
            expiresAt = now.plus(sessionTtl),
        )

    private fun save(session: ConversationSession, now: java.time.Instant): ConversationSession {
        val refreshed = session.withRefreshedTimestamps(now, now.plus(sessionTtl))
        sessions.save(refreshed)
        return refreshed
    }
}
