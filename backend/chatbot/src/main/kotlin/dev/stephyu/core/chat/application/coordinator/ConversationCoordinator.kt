package dev.stephyu.core.chat.application.coordinator

import dev.stephyu.core.chat.application.port.out.NlpAnalyzer
import dev.stephyu.core.chat.application.signal.ConversationSignalExtractor
import dev.stephyu.core.chat.domain.intent.IntentDecision
import dev.stephyu.core.chat.application.intent.decision.IntentDecisionEngine
import dev.stephyu.core.chat.application.signal.ConversationSignal
import dev.stephyu.core.chat.application.state.ProcessingMode
import dev.stephyu.core.chat.application.state.ConversationTurnContext
import dev.stephyu.core.chat.application.state.ConversationTurnResult
import dev.stephyu.core.chat.application.state.ConversationStateDispatcher
import dev.stephyu.core.chat.domain.conversation.ConversationAct
import dev.stephyu.core.chat.domain.session.ConversationSession
import dev.stephyu.core.chat.domain.intent.IntentName
import dev.stephyu.core.chat.domain.nlp.NlpAnalysis
import dev.stephyu.core.chat.domain.nlp.NlpAnalysisContext
import dev.stephyu.core.chat.domain.nlp.SlotName
import org.slf4j.LoggerFactory

/**
 * Coordinates a full chat turn from preprocessing to NLP analysis, intent routing, state handling, and reply assembly.
 */
class ConversationCoordinator(
    private val nlpAnalyzer: NlpAnalyzer,
    private val signalExtractor: ConversationSignalExtractor,
    private val intentDecisionEngine: IntentDecisionEngine,
    private val stateDispatcher: ConversationStateDispatcher,
) {
    private val logger = LoggerFactory.getLogger(ConversationCoordinator::class.java)

    /**
     * Processes one user message against the current in-memory session.
     */
    suspend fun handle(session: ConversationSession, message: String): ConversationTurnOutcome {
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

        val preprocessed = signalExtractor.preprocess(message)
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
        preprocessed: ConversationSignal,
    ): ConversationTurnOutcome {
        val analysis = analyze(preprocessed.businessText, session)
        val decision = intentDecisionEngine.resolve(analysis, session, preprocessed.businessText)
        logger.info(
            "Intent routing decision: sessionId={}, primaryIntent={}, alternatives={}, decision={}",
            session.id,
            analysis.intent.name,
            analysis.intent.alternatives,
            decision,
        )
        return when (decision) {
            is IntentDecision.Accept -> handleAcceptedIntent(
                session = session,
                preprocessed = preprocessed,
                analysis = analysis,
                resolvedIntent = decision.intent,
            )

            is IntentDecision.Clarify -> clarificationResult(
                session = session.withPendingDisambiguation(decision.primary, decision.secondary),
                preprocessed = preprocessed,
                primary = decision.primary,
                secondary = decision.secondary,
            )

            IntentDecision.Unknown -> unknownResult(
                session = session.clearPendingDisambiguation(),
                preprocessed = preprocessed,
            )
        }
    }

    private fun handleAcceptedIntent(
        session: ConversationSession,
        preprocessed: ConversationSignal,
        analysis: NlpAnalysis,
        resolvedIntent: IntentName,
    ): ConversationTurnOutcome {
        val handled = stateDispatcher.process(
            ConversationTurnContext(
                session = session.clearPendingDisambiguation(),
                intent = resolvedIntent,
                message = preprocessed.businessText,
                analysis = analysis,
                workflowCommand = preprocessed.workflowCommand,
                processingMode = ProcessingMode.PRIMARY,
            )
        )
        return resultFromState(preprocessed, handled, resolvedIntent)
    }

    private fun handleWorkflowCommand(
        session: ConversationSession,
        message: String,
        preprocessed: ConversationSignal,
    ): ConversationTurnOutcome {
        val workflowIntent = session.currentWorkflow?.ownerIntent ?: IntentName.UNKNOWN
        val handled = stateDispatcher.process(
            ConversationTurnContext(
                session = session,
                intent = workflowIntent,
                message = message,
                analysis = NlpAnalysis.unavailable,
                workflowCommand = preprocessed.workflowCommand,
                processingMode = ProcessingMode.PRIMARY,
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
        preprocessed: ConversationSignal,
        handled: ConversationTurnResult,
        intent: IntentName,
    ): ConversationTurnOutcome =
        ConversationTurnOutcome(
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

    private fun emptyMessageResult(session: ConversationSession): ConversationTurnOutcome =
        ConversationTurnOutcome(
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
    ): ConversationTurnOutcome =
        ConversationTurnOutcome(
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
        preprocessed: ConversationSignal,
        primary: IntentName,
        secondary: IntentName,
    ): ConversationTurnOutcome =
        ConversationTurnOutcome(
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
        preprocessed: ConversationSignal,
    ): ConversationTurnOutcome =
        ConversationTurnOutcome(
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

    private fun applyConversationActPrefix(preprocessed: ConversationSignal, reply: String): String =
        if (preprocessed.hasLeadingGreeting && !reply.startsWith("Hello.")) "Hello. $reply" else reply

    companion object {
        private const val RESTAURANT_DOMAIN = "restaurant"
    }
}

data class ConversationTurnOutcome(
    val session: ConversationSession,
    val intent: IntentName,
    val conversationAct: ConversationAct?,
    val reply: String,
    val slots: Map<SlotName, String>,
    val missingSlots: List<SlotName>,
    val completed: Boolean,
)


