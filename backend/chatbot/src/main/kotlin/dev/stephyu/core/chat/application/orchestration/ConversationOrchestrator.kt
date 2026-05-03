package dev.stephyu.core.chat.application.orchestration

import dev.stephyu.core.chat.application.port.out.NlpAnalyzer
import dev.stephyu.core.chat.application.service.ConversationActPreprocessor
import dev.stephyu.core.chat.application.service.IntentRoutingDecision
import dev.stephyu.core.chat.application.service.IntentResolver
import dev.stephyu.core.chat.application.service.PreprocessedMessage
import dev.stephyu.core.chat.application.state.ConversationStateInput
import dev.stephyu.core.chat.application.state.ConversationStateResult
import dev.stephyu.core.chat.application.state.ConversationStateMachine
import dev.stephyu.core.chat.domain.ConversationAct
import dev.stephyu.core.chat.domain.ConversationSession
import dev.stephyu.core.chat.domain.IntentName
import dev.stephyu.core.chat.domain.NlpAnalysis
import dev.stephyu.core.chat.domain.NlpAnalysisContext
import dev.stephyu.core.chat.domain.SlotName
import org.slf4j.LoggerFactory

class ConversationOrchestrator(
    private val nlpAnalyzer: NlpAnalyzer,
    private val conversationActPreprocessor: ConversationActPreprocessor,
    private val intentResolver: IntentResolver,
    private val stateMachine: ConversationStateMachine,
) {
    private val logger = LoggerFactory.getLogger(ConversationOrchestrator::class.java)

    suspend fun handle(session: ConversationSession, message: String): ConversationOrchestrationResult {
        logger.info(
            "Chat message received: sessionId={}, state={}, workflowIntent={}, message={}",
            session.id,
            session.state,
            session.currentWorkflow?.ownerIntent,
            message,
        )
        if (message.isBlank()) {
            return emptyMessageResult(session)
        }

        val preprocessed = conversationActPreprocessor.preprocess(message)
        logger.info(
            "Chat message preprocessed: sessionId={}, conversationAct={}, workflowCommand={}, businessText={}",
            session.id,
            preprocessed.conversationAct,
            preprocessed.workflowCommand,
            preprocessed.businessText,
        )
        return when {
            session.hasCurrentWorkflow() && preprocessed.workflowCommand != null ->
                handleWorkflowCommand(session, message.trim(), preprocessed)

            preprocessed.businessText.isBlank() && preprocessed.conversationAct != null ->
                conversationActResult(session, preprocessed.conversationAct)

            preprocessed.businessText.isBlank() ->
                emptyMessageResult(session)

            else -> handleBusinessMessage(session, preprocessed)
        }
    }

    private suspend fun handleBusinessMessage(
        session: ConversationSession,
        preprocessed: PreprocessedMessage,
    ): ConversationOrchestrationResult {
        val analysis = analyze(preprocessed.businessText, session)
        val decision = intentResolver.resolve(analysis, session, preprocessed.businessText)
        logger.info(
            "Intent routing decision: sessionId={}, primaryIntent={}, alternatives={}, decision={}",
            session.id,
            analysis.intent.name,
            analysis.intent.alternatives,
            decision,
        )
        return when (decision) {
            is IntentRoutingDecision.Accept -> handleAcceptedIntent(
                session = session,
                preprocessed = preprocessed,
                analysis = analysis,
                resolvedIntent = decision.intent,
            )

            is IntentRoutingDecision.Clarify -> clarificationResult(
                session = session.withPendingDisambiguation(decision.primary, decision.secondary),
                preprocessed = preprocessed,
                primary = decision.primary,
                secondary = decision.secondary,
            )

            IntentRoutingDecision.Unknown -> unknownResult(
                session = session.clearPendingDisambiguation(),
                preprocessed = preprocessed,
            )
        }
    }

    private fun handleAcceptedIntent(
        session: ConversationSession,
        preprocessed: PreprocessedMessage,
        analysis: NlpAnalysis,
        resolvedIntent: IntentName,
    ): ConversationOrchestrationResult {
        val handled = stateMachine.process(
            ConversationStateInput(
                session = session.clearPendingDisambiguation(),
                intent = resolvedIntent,
                message = preprocessed.businessText,
                analysis = analysis,
                workflowCommand = preprocessed.workflowCommand,
            )
        )
        return resultFromState(preprocessed, handled, resolvedIntent)
    }

    private fun handleWorkflowCommand(
        session: ConversationSession,
        message: String,
        preprocessed: PreprocessedMessage,
    ): ConversationOrchestrationResult {
        val workflowIntent = session.currentWorkflow?.ownerIntent ?: IntentName.UNKNOWN
        val handled = stateMachine.process(
            ConversationStateInput(
                session = session,
                intent = workflowIntent,
                message = message,
                analysis = NlpAnalysis.unavailable,
                workflowCommand = preprocessed.workflowCommand,
            )
        )
        return resultFromState(preprocessed, handled, workflowIntent)
    }

    private suspend fun analyze(message: String, session: ConversationSession): NlpAnalysis =
        runCatching {
            nlpAnalyzer.analyze(
                text = message,
                domain = RESTAURANT_DOMAIN,
                context = session.toNlpContext(),
            )
        }.onFailure { error ->
            logger.warn("NLP analyze failed: sessionId={}, message={}", session.id, message, error)
        }.getOrDefault(NlpAnalysis.unavailable)
            .also { analysis ->
                logger.info(
                    "NLP analysis: sessionId={}, intent={}, confidence={}, alternatives={}, entities={}",
                    session.id,
                    analysis.intent.name,
                    analysis.intent.confidence,
                    analysis.intent.alternatives,
                    analysis.entities.map { "${it.type}:${it.value}:${it.confidence}" },
                )
            }

    private fun resultFromState(
        preprocessed: PreprocessedMessage,
        handled: ConversationStateResult,
        intent: IntentName,
    ): ConversationOrchestrationResult =
        ConversationOrchestrationResult(
            session = handled.session,
            intent = handled.handledIntent ?: intent,
            conversationAct = preprocessed.conversationAct,
            reply = applyConversationActPrefix(preprocessed, handled.reply),
            slots = handled.slots,
            missingSlots = handled.missingSlots,
            completed = handled.completed,
        ).also { result ->
            logger.info(
                "Chat message handled: sessionId={}, resolvedIntent={}, handledIntent={}, nextState={}, missingSlots={}, completed={}",
                result.session.id,
                intent,
                result.intent,
                result.session.state,
                result.missingSlots,
                result.completed,
            )
        }

    private fun emptyMessageResult(session: ConversationSession): ConversationOrchestrationResult =
        ConversationOrchestrationResult(
            session = session,
            intent = IntentName.UNKNOWN,
            conversationAct = null,
            reply = emptyMessageReply(),
            slots = session.filledSlots(),
            missingSlots = session.missingSlots(),
            completed = false,
        )

    private fun conversationActResult(
        session: ConversationSession,
        conversationAct: ConversationAct,
    ): ConversationOrchestrationResult =
        ConversationOrchestrationResult(
            session = session,
            intent = IntentName.UNKNOWN,
            conversationAct = conversationAct,
            reply = conversationActReply(conversationAct),
            slots = session.filledSlots(),
            missingSlots = session.missingSlots(),
            completed = false,
        )

    private fun clarificationResult(
        session: ConversationSession,
        preprocessed: PreprocessedMessage,
        primary: IntentName,
        secondary: IntentName,
    ): ConversationOrchestrationResult =
        ConversationOrchestrationResult(
            session = session,
            intent = IntentName.UNKNOWN,
            conversationAct = preprocessed.conversationAct,
            reply = applyConversationActPrefix(preprocessed, clarificationReply(primary, secondary)),
            slots = session.filledSlots(),
            missingSlots = session.missingSlots(),
            completed = false,
        )

    private fun unknownResult(
        session: ConversationSession,
        preprocessed: PreprocessedMessage,
    ): ConversationOrchestrationResult =
        ConversationOrchestrationResult(
            session = session,
            intent = IntentName.UNKNOWN,
            conversationAct = preprocessed.conversationAct,
            reply = applyConversationActPrefix(preprocessed, unknownReply()),
            slots = session.filledSlots(),
            missingSlots = session.missingSlots(),
            completed = false,
        )

    private fun ConversationSession.toNlpContext(): NlpAnalysisContext =
        NlpAnalysisContext(
            currentIntent = currentWorkflow?.ownerIntent,
            previousIntent = previousIntent.takeIf { currentWorkflow != null },
            slotsFilled = currentWorkflow?.filledSlots() ?: emptyMap(),
            requiredSlots = currentWorkflow?.requiredSlotsForNlp() ?: emptyList(),
        )

    private fun emptyMessageReply(): String =
        "Please send a message so I can help you."

    private fun conversationActReply(conversationAct: ConversationAct): String = when (conversationAct) {
        ConversationAct.GREETING -> "Hello. How can I help you today?"
        ConversationAct.THANKS -> "You're welcome."
        ConversationAct.FAREWELL -> "Goodbye. See you soon."
    }

    private fun clarificationReply(primary: IntentName, secondary: IntentName): String =
        "Did you want ${clarificationLabel(primary)} or ${clarificationLabel(secondary)}?"

    private fun clarificationLabel(intent: IntentName): String = when (intent) {
        IntentName.MENU_REQUEST -> "menu options"
        IntentName.PRICING_REQUEST -> "pricing information"
        IntentName.LOCATION_REQUEST -> "location details"
        IntentName.CONTACT_REQUEST -> "contact details"
        IntentName.OPENING_HOURS -> "opening hours"
        else -> "help with your request"
    }

    private fun unknownReply(): String =
        "I did not understand that. I can help with reservations, opening hours, location, menu, prices, and contact details."

    private fun applyConversationActPrefix(preprocessed: PreprocessedMessage, reply: String): String =
        if (preprocessed.hasLeadingGreeting && !reply.startsWith("Hello.")) "Hello. $reply" else reply

    companion object {
        private const val RESTAURANT_DOMAIN = "restaurant"
    }
}

data class ConversationOrchestrationResult(
    val session: ConversationSession,
    val intent: IntentName,
    val conversationAct: ConversationAct?,
    val reply: String,
    val slots: Map<SlotName, String>,
    val missingSlots: List<SlotName>,
    val completed: Boolean,
)
