package dev.stephyu.core.chat.domain.nlp

import dev.stephyu.core.chat.domain.intent.IntentName

data class NlpAnalysis(
    val intent: NlpIntent,
    val entities: List<NlpEntity> = emptyList(),
    val intents: List<NlpIntent> = emptyList(),
    val utterance: NlpUtterance = NlpUtterance(
        kind = NlpUtteranceKind.UNKNOWN,
        confidence = 0.0,
        source = "missing",
    ),
    val warnings: List<String> = emptyList(),
) {
    fun rankedIntents(): List<NlpIntent> =
        intents.ifEmpty {
            listOf(intent) + intent.alternatives.map { (name, confidence) ->
                NlpIntent(
                    name = name,
                    confidence = confidence,
                    source = "alternative",
                )
            }
        }

    companion object {
        val unavailable = NlpAnalysis(
            intent = NlpIntent(
                name = IntentName.UNKNOWN,
                confidence = 0.0,
                source = "unavailable",
            ),
            utterance = NlpUtterance(
                kind = NlpUtteranceKind.UNKNOWN,
                confidence = 0.0,
                source = "unavailable",
            ),
        )
    }
}

data class NlpIntent(
    val name: IntentName,
    val confidence: Double,
    val source: String,
    val alternatives: Map<IntentName, Double> = emptyMap(),
)

data class NlpEntity(
    val type: SlotName,
    val value: String,
    val confidence: Double,
    val source: String,
    val rawValue: String = value,
    val resolution: String? = null,
    val normalizationStatus: NlpEntityNormalizationStatus = NlpEntityNormalizationStatus.RAW_ONLY,
)

data class NlpUtterance(
    val kind: NlpUtteranceKind,
    val confidence: Double,
    val source: String,
)

enum class NlpUtteranceKind(val wireName: String) {
    BUSINESS_QUERY("business_query"),
    SMALL_TALK("small_talk"),
    VAGUE_FOLLOW_UP("vague_follow_up"),
    CLARIFICATION_REQUEST("clarification_request"),
    FRUSTRATION("frustration"),
    OUT_OF_DOMAIN("out_of_domain"),
    AMBIGUOUS("ambiguous"),
    UNKNOWN("unknown");

    companion object {
        fun fromWireName(value: String?): NlpUtteranceKind =
            entries.firstOrNull { it.wireName == value } ?: UNKNOWN
    }
}

enum class NlpEntityNormalizationStatus(val wireName: String) {
    NORMALIZED("normalized"),
    RAW_ONLY("raw_only"),
    AMBIGUOUS("ambiguous"),
    FAILED("failed");

    companion object {
        fun fromWireName(value: String?): NlpEntityNormalizationStatus =
            entries.firstOrNull { it.wireName == value } ?: RAW_ONLY
    }
}

data class NlpAnalysisContext(
    val currentIntent: IntentName?,
    val previousIntent: IntentName?,
    val slotsFilled: Map<SlotName, String>,
    val requiredSlots: List<SlotName>,
)


