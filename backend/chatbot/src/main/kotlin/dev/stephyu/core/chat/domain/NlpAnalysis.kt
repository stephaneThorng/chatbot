package dev.stephyu.core.chat.domain

data class NlpAnalysis(
    val intent: NlpIntent,
    val entities: List<NlpEntity> = emptyList(),
) {
    companion object {
        val unavailable = NlpAnalysis(
            intent = NlpIntent(
                name = IntentName.UNKNOWN,
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
)

data class NlpAnalysisContext(
    val currentIntent: IntentName?,
    val previousIntent: IntentName?,
    val slotsFilled: Map<SlotName, String>,
    val requiredSlots: List<SlotName>,
)
