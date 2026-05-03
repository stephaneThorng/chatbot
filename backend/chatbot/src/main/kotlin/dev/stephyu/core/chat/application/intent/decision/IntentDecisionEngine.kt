package dev.stephyu.core.chat.application.intent.decision

import dev.stephyu.core.chat.application.intent.policy.IntentCategory
import dev.stephyu.core.chat.application.intent.catalog.IntentCatalog
import dev.stephyu.core.chat.domain.intent.IntentDecision
import dev.stephyu.core.chat.domain.session.ConversationSession
import dev.stephyu.core.chat.domain.intent.IntentName
import dev.stephyu.core.chat.domain.nlp.NlpAnalysis

/**
 * Combines NLP evidence with session state and intent policies to select the next business routing decision.
 */
class IntentDecisionEngine(
    private val intentCatalog: IntentCatalog,
    private val confidenceThreshold: Double = 0.6,
    private val clarificationConfidenceThreshold: Double = 0.38,
    private val clarificationMarginThreshold: Double = 0.14,
) {
    /**
     * Resolves the next routing decision for the current message.
     */
    fun resolve(
        analysis: NlpAnalysis,
        session: ConversationSession,
        message: String,
    ): IntentDecision {
        val normalized = message.trim().lowercase()
        if (!session.hasCurrentWorkflow() && shouldIgnoreIdleMessage(normalized)) {
            return IntentDecision.Unknown
        }
        if (session.hasCurrentWorkflow()) {
            return resolveForActiveWorkflow(analysis, session, normalized)
        }

        resolvePendingDisambiguation(analysis, session, normalized)?.let { return it }

        if (isTopicContinuation(normalized)) {
            val topicIntent = session.lastResolvedInformationalIntent
            if (topicIntent != null && supportsTopicContinuation(topicIntent)) {
                return IntentDecision.Accept(topicIntent, source = "topic_memory")
            }
            return IntentDecision.Unknown
        }

        val primary = analysis.intent
        val secondary = topAlternative(analysis)
        if (primary.name == IntentName.UNKNOWN || primary.confidence < confidenceThreshold) {
            return clarifyOrUnknown(primary.name, secondary, analysis, normalized)
        }
        if (isUnsupportedStandaloneContent(normalized, analysis)) {
            return IntentDecision.Unknown
        }
        if (shouldClarify(primary.name, primary.confidence, secondary, analysis)) {
            return IntentDecision.Clarify(
                primary = primary.name,
                secondary = secondary!!.first,
            )
        }
        return IntentDecision.Accept(primary.name, source = "nlp")
    }

    private fun resolveForActiveWorkflow(
        analysis: NlpAnalysis,
        session: ConversationSession,
        normalized: String,
    ): IntentDecision {
        val workflowIntent = session.currentWorkflow?.ownerIntent ?: return IntentDecision.Unknown
        if (isWorkflowScopedMessage(normalized)) {
            return IntentDecision.Accept(workflowIntent, source = "workflow_scope")
        }
        if (allowsDuringWorkflow(analysis.intent.name) && analysis.intent.confidence >= confidenceThreshold) {
            return IntentDecision.Accept(analysis.intent.name, source = "nlp")
        }
        return IntentDecision.Accept(workflowIntent, source = "workflow_owner")
    }

    private fun resolvePendingDisambiguation(
        analysis: NlpAnalysis,
        session: ConversationSession,
        normalized: String,
    ): IntentDecision? {
        val pending = session.pendingDisambiguation ?: return null
        if (pending.candidates.isEmpty()) return null

        explicitDisambiguationChoice(normalized, pending.candidates)?.let { chosen ->
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

    private fun clarifyOrUnknown(
        primary: IntentName,
        secondary: Pair<IntentName, Double>?,
        analysis: NlpAnalysis,
        normalized: String,
    ): IntentDecision {
        if (isUnsupportedStandaloneContent(normalized, analysis)) {
            return IntentDecision.Unknown
        }
        if (shouldClarify(primary, analysis.intent.confidence, secondary, analysis)) {
            return IntentDecision.Clarify(
                primary = primary,
                secondary = secondary!!.first,
            )
        }
        return IntentDecision.Unknown
    }

    private fun shouldClarify(
        primary: IntentName,
        primaryConfidence: Double,
        secondary: Pair<IntentName, Double>?,
        analysis: NlpAnalysis,
    ): Boolean {
        if (!isClarifiable(primary)) return false
        if (secondary == null || !isClarifiable(secondary.first)) return false
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
        analysis.intent.alternatives
            .filterKeys { it != IntentName.UNKNOWN }
            .maxByOrNull { it.value }
            ?.let { it.key to it.value }

    private fun shouldIgnoreIdleMessage(message: String): Boolean =
        isLowSignalMessage(message) ||
            isStandaloneControlMessage(message) ||
            isPersonalSmallTalk(message) ||
            isAmbiguousStandaloneToken(message)

    private fun isLowSignalMessage(message: String): Boolean {
        if (message.isBlank()) return true
        val token = message.split(WHITESPACE).singleOrNull() ?: return false
        val lettersOnly = token.filter { it.isLetter() }
        return lettersOnly.length in 1..2
    }

    private fun isStandaloneControlMessage(message: String): Boolean {
        val token = message.split(WHITESPACE).singleOrNull() ?: return false
        return token in IDLE_CONTROL_TOKENS
    }

    private fun isPersonalSmallTalk(message: String): Boolean =
        PERSONAL_SMALL_TALK_PATTERNS.any { it.containsMatchIn(message) }

    private fun isAmbiguousStandaloneToken(message: String): Boolean {
        val token = message.split(WHITESPACE).singleOrNull() ?: return false
        return token.all { it.isLetter() } && token.length in 3..5
    }

    private fun isWorkflowScopedMessage(message: String): Boolean {
        val token = message.split(WHITESPACE).singleOrNull() ?: return false
        return token in WORKFLOW_SCOPED_TOKENS
    }

    private fun isTopicContinuation(message: String): Boolean =
        TOPIC_CONTINUATION_PATTERNS.any { it.matches(message) }

    private fun isUnsupportedStandaloneContent(message: String, analysis: NlpAnalysis): Boolean {
        val tokens = message.split(WHITESPACE).filter { it.isNotBlank() }
        if (tokens.size != 1) return false
        if (analysis.entities.isNotEmpty()) return false
        val token = tokens.single().filter { it.isLetter() }
        if (token.length < 3) return false
        return isInformational(analysis.intent.name) || analysis.intent.name == IntentName.UNKNOWN
    }

    private fun explicitDisambiguationChoice(
        message: String,
        candidates: List<IntentName>,
    ): IntentName? =
        candidates.firstOrNull { candidate ->
            intentCatalog.findIntentPolicy(candidate).disambiguationLabels.any { label ->
                Regex("""\b${Regex.escape(label)}\b""").containsMatchIn(message)
            }
        }

    private fun isClarifiable(intent: IntentName): Boolean =
        intentCatalog.findIntentPolicy(intent).clarifiable

    private fun isInformational(intent: IntentName): Boolean =
        intentCatalog.findIntentPolicy(intent).category == IntentCategory.INFORMATIONAL

    private fun supportsTopicContinuation(intent: IntentName): Boolean =
        intentCatalog.findIntentPolicy(intent).supportsTopicContinuation

    private fun allowsDuringWorkflow(intent: IntentName): Boolean =
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
        private val IDLE_CONTROL_TOKENS = WORKFLOW_SCOPED_TOKENS + setOf(
            "hello",
            "hi",
            "hey",
            "thanks",
            "bye",
        )
        private val TOPIC_CONTINUATION_PATTERNS = listOf(
            Regex("""(?i)^\s*what else\s*\??\s*$"""),
            Regex("""(?i)^\s*anything else\s*\??\s*$"""),
            Regex("""(?i)^\s*anything more\s*\??\s*$"""),
            Regex("""(?i)^\s*other options\s*\??\s*$"""),
            Regex("""(?i)^\s*what about the rest\s*\??\s*$"""),
        )
        private val PERSONAL_SMALL_TALK_PATTERNS = listOf(
            Regex("""(?i)^\s*how\s+am\s+i\b"""),
            Regex("""(?i)^\s*how\s+are\s+you\b"""),
            Regex("""(?i)^\s*i\s+am\b"""),
            Regex("""(?i)^\s*i['â€™]?m\b"""),
            Regex("""(?i)^\s*i\s+very\b"""),
        )
    }
}


