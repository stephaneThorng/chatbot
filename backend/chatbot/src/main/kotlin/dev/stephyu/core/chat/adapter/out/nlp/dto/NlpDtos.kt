package dev.stephyu.core.chat.adapter.out.nlp.dto

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class NlpAnalysisRequestDto(
    val text: String,
    val domain: String,
    val context: NlpAnalysisContextDto?,
)

@Serializable
data class NlpAnalysisContextDto(
    @SerialName("current_intent")
    val currentIntent: String? = null,
    @SerialName("previous_intent")
    val previousIntent: String? = null,
    @SerialName("slots_filled")
    val slotsFilled: NlpContextSlotsDto? = null,
    @SerialName("required_slots")
    val requiredSlots: List<String> = emptyList(),
)

@Serializable
data class NlpContextSlotsDto(
    val date: String? = null,
    val time: String? = null,
    val people: String? = null,
    val name: String? = null,
    val phone: String? = null,
    val email: String? = null,
    @SerialName("menu_item")
    val menuItem: String? = null,
    @SerialName("price_item")
    val priceItem: String? = null,
    val location: String? = null,
)

@Serializable
data class NlpAnalysisResponseDto(
    val intent: NlpIntentDto,
    val intents: List<NlpIntentCandidateDto> = emptyList(),
    val utterance: NlpUtteranceDto = NlpUtteranceDto(
        kind = "unknown",
        confidence = 0.0,
        source = "missing",
    ),
    val entities: List<NlpEntityDto> = emptyList(),
    val warnings: List<String> = emptyList(),
)

@Serializable
data class NlpIntentDto(
    val name: String,
    val confidence: Double,
    val source: String,
    val alternatives: Map<String, Double> = emptyMap(),
)

@Serializable
data class NlpIntentCandidateDto(
    val name: String,
    val confidence: Double,
    val source: String,
    val reason: String? = null,
)

@Serializable
data class NlpUtteranceDto(
    val kind: String,
    val confidence: Double,
    val source: String,
)

@Serializable
data class NlpEntityDto(
    val type: String,
    @SerialName("raw_value")
    val rawValue: String? = null,
    val value: String,
    val confidence: Double,
    val source: String,
    val resolution: String? = null,
    @SerialName("normalization_status")
    val normalizationStatus: String = "raw_only",
)


