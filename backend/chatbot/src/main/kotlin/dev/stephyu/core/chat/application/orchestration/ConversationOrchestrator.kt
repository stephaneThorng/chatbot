package dev.stephyu.core.chat.application.orchestration

import dev.stephyu.core.chat.application.port.out.NlpAnalyzer
import dev.stephyu.core.chat.application.service.ConversationActPreprocessor
import dev.stephyu.core.chat.application.service.IntentResolver
import dev.stephyu.core.chat.application.service.ReplyComposer
import dev.stephyu.core.chat.application.state.ChatStateInput
import dev.stephyu.core.chat.application.state.ChatStateMachine
import dev.stephyu.core.chat.domain.ConversationAct
import dev.stephyu.core.chat.domain.ConversationSession
import dev.stephyu.core.chat.domain.IntentName
import dev.stephyu.core.chat.domain.NlpAnalysis
import dev.stephyu.core.chat.domain.NlpAnalysisContext
import dev.stephyu.core.chat.domain.SlotName

class ChatMessageOrchestrator(
    private val nlpAnalyzer: NlpAnalyzer,
    private val conversationActPreprocessor: ConversationActPreprocessor,
    private val intentResolver: IntentResolver,
    private val stateMachine: ChatStateMachine,
    private val replies: ReplyComposer,
) {
    suspend fun handle(session: ConversationSession, message: String): ChatOrchestrationResult {
        if (message.isBlank()) {
            return ChatOrchestrationResult(
                session = session,
                intent = IntentName.UNKNOWN,
                conversationAct = null,
                reply = replies.emptyMessageReply(),
                slots = session.currentSlots(),
                missingSlots = session.missingCurrentSlots(),
                completed = false,
            )
        }

        val preprocessed = conversationActPreprocessor.preprocess(message)
        if (preprocessed.businessText.isBlank() && preprocessed.conversationAct != null) {
            return ChatOrchestrationResult(
                session = session,
                intent = IntentName.UNKNOWN,
                conversationAct = preprocessed.conversationAct,
                reply = replies.conversationActReply(preprocessed.conversationAct),
                slots = session.currentSlots(),
                missingSlots = session.missingCurrentSlots(),
                completed = false,
            )
        }
        if (preprocessed.businessText.isBlank()) {
            return ChatOrchestrationResult(
                session = session,
                intent = IntentName.UNKNOWN,
                conversationAct = null,
                reply = replies.emptyMessageReply(),
                slots = session.currentSlots(),
                missingSlots = session.missingCurrentSlots(),
                completed = false,
            )
        }

        val analysis = runCatching {
            nlpAnalyzer.analyze(
                text = preprocessed.businessText,
                domain = RESTAURANT_DOMAIN,
                context = session.toNlpContext(),
            )
        }.getOrDefault(NlpAnalysis.unavailable)

        val intent = intentResolver.resolve(analysis, session)
        val handled = stateMachine.process(
            ChatStateInput(
                session = session,
                intent = intent,
                message = preprocessed.businessText,
                analysis = analysis,
            )
        )

        return ChatOrchestrationResult(
            session = handled.session,
            intent = intent,
            conversationAct = preprocessed.conversationAct,
            reply = replies.applyConversationActPrefix(preprocessed, handled.reply),
            slots = handled.slots,
            missingSlots = handled.missingSlots,
            completed = handled.completed,
        )
    }

    private fun ConversationSession.toNlpContext(): NlpAnalysisContext =
        NlpAnalysisContext(
            currentIntent = currentWorkflow?.name?.toIntentName(),
            previousIntent = previousIntent.takeIf { currentWorkflow != null },
            slotsFilled = currentWorkflow?.wireSlots() ?: emptyMap(),
            requiredSlots = currentWorkflow?.requiredSlotsForNlp() ?: emptyList(),
        )

    companion object {
        private const val RESTAURANT_DOMAIN = "restaurant"
    }
}

data class ChatOrchestrationResult(
    val session: ConversationSession,
    val intent: IntentName,
    val conversationAct: ConversationAct?,
    val reply: String,
    val slots: Map<SlotName, String>,
    val missingSlots: List<SlotName>,
    val completed: Boolean,
)
