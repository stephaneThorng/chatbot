package dev.stephyu.core.chat.domain.workflow

import java.time.LocalDate
import java.time.LocalTime

sealed interface RequirementValue {
    val raw: String
    val displayValue: String
}

data class TextRequirementValue(
    override val raw: String,
    override val displayValue: String,
) : RequirementValue

data class DateRequirementValue(
    override val raw: String,
    val value: LocalDate,
    override val displayValue: String,
) : RequirementValue

data class TimeRequirementValue(
    override val raw: String,
    val value: LocalTime,
    override val displayValue: String,
) : RequirementValue

data class IntegerRequirementValue(
    override val raw: String,
    val value: Int,
    override val displayValue: String,
) : RequirementValue

data class BooleanRequirementValue(
    override val raw: String,
    val value: Boolean,
    override val displayValue: String,
) : RequirementValue