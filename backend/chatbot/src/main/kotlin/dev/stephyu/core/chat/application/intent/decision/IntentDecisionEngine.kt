package dev.stephyu.core.chat.application.intent.decision

import dev.stephyu.core.chat.application.intent.catalog.IntentCatalog
import dev.stephyu.core.chat.domain.intent.IntentDecision
import dev.stephyu.core.chat.domain.intent.IntentName
import dev.stephyu.core.chat.domain.nlp.NlpAnalysis
import dev.stephyu.core.chat.domain.nlp.NlpUtteranceKind
import dev.stephyu.core.chat.domain.session.ConversationSession
import dev.stephyu.core.chat.domain.session.PendingDisambiguation

/**
 * Chooses the next business routing decision from NLP evidence and conversation state.
 *
 * This class does not classify user text locally. Backend-owned text signals such as
 * conversation acts and workflow commands are extracted before this decision step.
 */
class IntentDecisionEngine(
    private val intentCatalog: IntentCatalog,
    private val confidenceThreshold: Double = 0.6,
    private val clarificationConfidenceThreshold: Double = 0.38,
    private val clarificationMarginThreshold: Double = 0.14,
) {
    /**
     * Resolves the next routing Intent decision for the current message.
     */
    fun resolve(
        analysis: NlpAnalysis,
        session: ConversationSession,
        processedText: String,
    ): IntentDecision {
        val normalizedText = processedText.trim().lowercase()
        if (!session.hasCurrentWorkflow() && normalizedText.isBlank()) {
            return IntentDecision.Unknown
        }
        if (session.hasCurrentWorkflow()) {
            return session.currentWorkflow
                ?.ownerIntent
                ?.let { workflowIntent -> resolveForActiveWorkflow(analysis, workflowIntent, normalizedText) }
                ?: IntentDecision.Unknown
        }

        if (session.pendingDisambiguation != null) {
            resolvePendingDisambiguation(analysis, session.pendingDisambiguation, normalizedText)
                ?.let { return it }
        }

        resolveUtteranceSignal(analysis, session)
            ?.let { return it }

        val primary = analysis.intent
        val secondary = topAlternative(analysis)

        if (secondary != null) {
            clarifyDecision(primary.name, primary.confidence, secondary, analysis)
                ?.let { return it }
        }

        if (primary.name == IntentName.UNKNOWN || primary.confidence < confidenceThreshold) {
            return IntentDecision.Unknown
        }
        return IntentDecision.Accept(primary.name, source = "nlp")
    }

    private fun resolveUtteranceSignal(
        analysis: NlpAnalysis,
        session: ConversationSession,
    ): IntentDecision? =
        when (analysis.utterance.kind) {
            NlpUtteranceKind.VAGUE_FOLLOW_UP -> resolveVagueFollowUp(session)
            NlpUtteranceKind.SMALL_TALK,
            NlpUtteranceKind.FRUSTRATION,
            NlpUtteranceKind.OUT_OF_DOMAIN,
            NlpUtteranceKind.UNKNOWN -> IntentDecision.Unknown
            NlpUtteranceKind.AMBIGUOUS -> resolveAmbiguousUtterance(analysis)
            NlpUtteranceKind.BUSINESS_QUERY,
            NlpUtteranceKind.CLARIFICATION_REQUEST -> null
        }

    private fun resolveVagueFollowUp(session: ConversationSession): IntentDecision {
        val topicIntent = session.lastResolvedInformationalIntent
        return if (topicIntent != null && isTopicContinuationSupported(topicIntent)) {
            IntentDecision.Accept(topicIntent, source = "utterance_topic_memory")
        } else {
            IntentDecision.Unknown
        }
    }

    private fun resolveAmbiguousUtterance(analysis: NlpAnalysis): IntentDecision {
        val primary = analysis.intent
        val secondary = topAlternative(analysis) ?: return IntentDecision.Unknown
        return clarifyDecision(primary.name, primary.confidence, secondary, analysis)
            ?: IntentDecision.Unknown
    }

    private fun resolveForActiveWorkflow(
        analysis: NlpAnalysis,
        workflowIntent: IntentName,
        normalizedText: String,
    ): IntentDecision {
        if (isWorkflowScopedMessage(normalizedText)) {
            return IntentDecision.Accept(workflowIntent, source = "workflow_scope")
        }
        if (isAllowedDuringWorkflow(analysis.intent.name) && analysis.intent.confidence >= confidenceThreshold) {
            return IntentDecision.Accept(analysis.intent.name, source = "nlp")
        }
        return IntentDecision.Accept(workflowIntent, source = "workflow_owner")
    }

    private fun resolvePendingDisambiguation(
        analysis: NlpAnalysis,
        pending: PendingDisambiguation,
        normalizedText: String,
    ): IntentDecision? {
        if (pending.candidates.isEmpty()) return null

        explicitDisambiguationChoice(normalizedText, pending.candidates)?.let { chosen ->
            return IntentDecision.Accept(chosen, source = "pending_disambiguation")
        }
        if (analysis.intent.name in pending.candidates && analysis.intent.confidence >= confidenceThreshold) {
            return IntentDecision.Accept(analysis.intent.name, source = "nlp")
        }
        if (analysis.intent.name != IntentName.UNKNOWN && analysis.intent.confidence >= confidenceThreshold) {
            return IntentDecision.Accept(analysis.intent.name, source = "nlp")
        }
        return IntentDecision.Clarify(
            primary = pending.candidates.first(),
            secondary = pending.candidates.drop(1).firstOrNull() ?: pending.candidates.first(),
        )
    }

    private fun clarifyDecision(
        primary: IntentName,
        primaryConfidence: Double,
        secondary: Pair<IntentName, Double>,
        analysis: NlpAnalysis,
    ): IntentDecision.Clarify? =
        if (shouldClarify(primary, primaryConfidence, secondary, analysis)) {
            IntentDecision.Clarify(
                primary = primary,
                secondary = secondary.first,
            )
        } else {
            null
        }

    private fun shouldClarify(
        primary: IntentName,
        primaryConfidence: Double,
        secondary: Pair<IntentName, Double>,
        analysis: NlpAnalysis,
    ): Boolean {
        if (!isClarifiable(primary)) return false
        if (!isClarifiable(secondary.first)) return false
        if (hasEntitySupport(primary, analysis) || hasEntitySupport(secondary.first, analysis)) return false
        return primaryConfidence >= clarificationConfidenceThreshold &&
                (primaryConfidence - secondary.second) <= clarificationMarginThreshold
    }

    private fun hasEntitySupport(intent: IntentName, analysis: NlpAnalysis): Boolean {
        val supported = intentCatalog.findIntentPolicy(intent).entitySupport
        if (supported.isEmpty()) return false
        val entityTypes = analysis.entities
            .filter { it.confidence >= ENTITY_SUPPORT_THRESHOLD }
            .map { it.type }
            .toSet()
        return entityTypes.any { it in supported }
    }

    private fun topAlternative(analysis: NlpAnalysis): Pair<IntentName, Double>? =
        analysis.rankedIntents()
            .dropWhile { it.name == analysis.intent.name }
            .firstOrNull { it.name != IntentName.UNKNOWN }
            ?.let { it.name to it.confidence }

    private fun isWorkflowScopedMessage(normalizedText: String): Boolean {
        val token = normalizedText
            .split(WHITESPACE)
            .singleOrNull()
            ?: return false
        return token in WORKFLOW_SCOPED_TOKENS
    }

    private fun explicitDisambiguationChoice(
        message: String,
        candidates: List<IntentName>,
    ): IntentName? =
        candidates.firstOrNull { candidate ->
            intentCatalog
                .findIntentPolicy(candidate)
                .disambiguationLabels
                .any { label ->
                    Regex("""\b${Regex.escape(label)}\b""")
                    .containsMatchIn(message)
                }
        }

    private fun isClarifiable(intent: IntentName): Boolean =
        intentCatalog.findIntentPolicy(intent).clarifiable

    private fun isTopicContinuationSupported(intent: IntentName): Boolean =
        intentCatalog.findIntentPolicy(intent).supportsTopicContinuation

    private fun isAllowedDuringWorkflow(intent: IntentName): Boolean =
        intentCatalog.findIntentPolicy(intent).allowDuringWorkflow

    companion object {
        private const val ENTITY_SUPPORT_THRESHOLD = 0.5
        private val WHITESPACE = Regex("""\s+""")
        private val WORKFLOW_SCOPED_TOKENS = setOf(
            "yes",
            "y",
            "no",
            "n",
            "ok",
            "okay",
            "sure",
            "why",
            "what",
            "how",
            "when",
        )
    }
}
