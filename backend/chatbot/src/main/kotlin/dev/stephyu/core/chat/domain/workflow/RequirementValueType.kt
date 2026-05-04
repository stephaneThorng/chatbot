package dev.stephyu.core.chat.domain.workflow

import dev.stephyu.core.chat.domain.nlp.SlotName
import java.time.LocalDate
import java.time.LocalTime

data class RequirementParsingContext(
    val today: LocalDate,
    val minPartySize: Int = 1,
    val maxPartySize: Int = 12,
    val earliestReservationTime: LocalTime = LocalTime.of(11, 30),
    val latestReservationTime: LocalTime = LocalTime.of(23, 30),
)

sealed interface RequirementParsingResult {
    data class Valid(val value: RequirementValue) : RequirementParsingResult
    data class Invalid(val message: String) : RequirementParsingResult
    data object NotMatched : RequirementParsingResult
}

interface RequirementValueType {
    val acceptedEntities: Set<SlotName>

    fun parse(raw: String, context: RequirementParsingContext): RequirementParsingResult
}