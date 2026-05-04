package dev.stephyu.core.chat.application.coordinator

import dev.stephyu.core.chat.application.port.out.NlpAnalyzer
import dev.stephyu.core.chat.application.signal.ConversationSignalExtractor
import dev.stephyu.core.chat.domain.intent.IntentDecision
import dev.stephyu.core.chat.application.intent.decision.IntentDecisionEngine
import dev.stephyu.core.chat.application.signal.ConversationSignal
import dev.stephyu.core.chat.application.state.ProcessingMode
import dev.stephyu.core.chat.application.state.StateHandlerInput
import dev.stephyu.core.chat.application.state.StateHandlerResult
import dev.stephyu.core.chat.application.state.ConversationStateDispatcher
import dev.stephyu.core.chat.domain.conversation.ConversationAct
import dev.stephyu.core.chat.domain.session.ConversationSession
import dev.stephyu.core.chat.domain.intent.IntentName
import dev.stephyu.core.chat.domain.nlp.NlpAnalysis
import dev.stephyu.core.chat.domain.nlp.NlpAnalysisContext
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

    companion object {
        private const val RESTAURANT_DOMAIN = "restaurant"
    }

    private val logger = LoggerFactory.getLogger(ConversationCoordinator::class.java)

    /**
     * Processes one user message against the current in-memory session.
     */
    suspend fun handle(session: ConversationSession, message: String): StateHandlerResult {

        if (message.isBlank()) {
            return emptyMessageResult(session)
        }

        val conversationSignal = signalExtractor.preprocess(message)

        return when {
            session.hasCurrentWorkflow() && conversationSignal.workflowCommand != null ->
                handleWorkflowCommand(session, conversationSignal)

            conversationSignal.processedText.isBlank() && conversationSignal.conversationAct != null ->
                conversationActResult(session, conversationSignal.conversationAct)

            conversationSignal.processedText.isBlank() ->
                emptyMessageResult(session)

            else -> handleBusinessMessage(session, conversationSignal)
        }
    }

    private suspend fun handleBusinessMessage(
        session: ConversationSession,
        conversationSignal: ConversationSignal,
    ): StateHandlerResult {
        val analysis = analyze(conversationSignal.processedText, session.id, session.toNlpContext())
        return when (val decision = intentDecisionEngine.resolve(analysis, session, conversationSignal.processedText)) {
            is IntentDecision.Accept -> {
                logger.debug(
                    "Intent accepted: sessionId={}, intent={}, utteranceKind={}",
                    session.id,
                    decision.intent,
                    analysis.utterance.kind,
                )
                val stateHandlerResult = stateDispatcher.process(
                    StateHandlerInput(
                        session = session.clearPendingDisambiguation(),
                        intent = decision.intent,
                        processedText = conversationSignal.processedText,
                        analysis = analysis,
                        workflowCommand = conversationSignal.workflowCommand,
                        processingMode = ProcessingMode.PRIMARY,
                    )
                )
                resultFromState(conversationSignal, stateHandlerResult, decision.intent)
            }

            is IntentDecision.Clarify -> {
                logger.debug(
                    "Intent clarification required: sessionId={}, primary={}, secondary={}, utteranceKind={}",
                    session.id,
                    decision.primary,
                    decision.secondary,
                    analysis.utterance.kind,
                )
                StateHandlerResult(
                    updatedSession = session.withPendingDisambiguation(decision.primary, decision.secondary),
                    reply = applyConversationActPrefix(conversationSignal.hasLeadingGreeting, clarificationReply(decision.primary, decision.secondary)),
                    conversationAct = conversationSignal.conversationAct,
                    handledIntentOverride = IntentName.UNKNOWN,
                )
            }

            IntentDecision.Unknown -> {
                logger.debug(
                    "Intent resolved to unknown: sessionId={}, utteranceKind={}, topIntent={}, topConfidence={}",
                    session.id,
                    analysis.utterance.kind,
                    analysis.intent.name,
                    analysis.intent.confidence,
                )
                StateHandlerResult(
                    updatedSession = session.clearPendingDisambiguation(),
                    reply = applyConversationActPrefix(conversationSignal.hasLeadingGreeting, unknownReply()),
                    conversationAct = conversationSignal.conversationAct,
                    handledIntentOverride = IntentName.UNKNOWN,
                )
            }
        }
    }

    private fun handleWorkflowCommand(
        session: ConversationSession,
        conversationSignal: ConversationSignal,
    ): StateHandlerResult {
        val workflowIntent = session.currentWorkflow?.ownerIntent ?: IntentName.UNKNOWN
        val handled = stateDispatcher.process(
            StateHandlerInput(
                session = session,
                intent = workflowIntent,
                processedText = conversationSignal.rawText,
                analysis = NlpAnalysis.unavailable,
                workflowCommand = conversationSignal.workflowCommand,
                processingMode = ProcessingMode.PRIMARY,
            )
        )
        return resultFromState(conversationSignal, handled, workflowIntent)
    }

    private suspend fun analyze(processedText: String, sessionId: String, nlpContext: NlpAnalysisContext): NlpAnalysis =
        runCatching {
            nlpAnalyzer.analyze(
                text = processedText,
                domain = RESTAURANT_DOMAIN,
                context = nlpContext,
            )
        }.onFailure { error ->
            logger.warn("NLP analyze failed: sessionId={}, message={}", sessionId, processedText, error)
        }.getOrDefault(NlpAnalysis.unavailable)
            .also { analysis ->
                logger.info(
                    "NLP analysis: sessionId={}, intent={}, confidence={}, alternatives={}, entities={}",
                    sessionId,
                    analysis.intent.name,
                    analysis.intent.confidence,
                    analysis.intent.alternatives,
                    analysis.entities.map { "${it.type}:${it.value}:${it.confidence}" },
                )
            }

    private fun resultFromState(
        conversationSignal: ConversationSignal,
        handled: StateHandlerResult,
        intent: IntentName,
    ): StateHandlerResult =
        handled.copy(
            updatedSession = handled.updatedSession,
            conversationAct = conversationSignal.conversationAct,
            reply = applyConversationActPrefix(conversationSignal.hasLeadingGreeting, handled.reply),
            completed = handled.completed,
            handledIntentOverride = handled.handledIntent ?: intent,
            slotSnapshot = handled.slotSnapshot,
            missingSlotSnapshot = handled.missingSlotSnapshot,
        ).also { result ->
            logger.info(
                "Chat message handled: sessionId={}, resolvedIntent={}, handledIntent={}, nextState={}, workflowOwner={}, missingSlots={}, completed={}",
                result.updatedSession.id,
                intent,
                result.handledIntent,
                result.updatedSession.state,
                result.updatedSession.currentWorkflow?.ownerIntent,
                result.missingSlots,
                result.completed,
            )
        }

    private fun emptyMessageResult(session: ConversationSession): StateHandlerResult =
        StateHandlerResult(
            updatedSession = session,
            reply = emptyMessageReply(),
            handledIntentOverride = IntentName.UNKNOWN,
        )

    private fun conversationActResult(
        session: ConversationSession,
        conversationAct: ConversationAct,
    ): StateHandlerResult =
        StateHandlerResult(
            updatedSession = session,
            reply = conversationActReply(conversationAct),
            conversationAct = conversationAct,
            handledIntentOverride = IntentName.UNKNOWN,
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

    private fun applyConversationActPrefix(hasLeadingGreeting: Boolean, reply: String): String =
        if (hasLeadingGreeting && !reply.startsWith("Hello.")) "Hello. $reply" else reply
}


