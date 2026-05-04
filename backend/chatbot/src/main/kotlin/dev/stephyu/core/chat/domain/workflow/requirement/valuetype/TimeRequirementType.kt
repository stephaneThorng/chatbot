package dev.stephyu.core.chat.domain.workflow.requirement.valuetype

import dev.stephyu.core.chat.domain.nlp.SlotName
import dev.stephyu.core.chat.domain.workflow.RequirementParsingContext
import dev.stephyu.core.chat.domain.workflow.RequirementParsingResult
import dev.stephyu.core.chat.domain.workflow.RequirementValueType
import dev.stephyu.core.chat.domain.workflow.TimeRequirementValue
import java.time.LocalTime
import java.util.Locale

data object TimeRequirementType : RequirementValueType {
    override val acceptedEntities: Set<SlotName> = setOf(SlotName.TIME)

    override fun parse(raw: String, context: RequirementParsingContext): RequirementParsingResult {
        val meridiemMatch = MERIDIEM_TIME_PATTERN.find(raw)
        if (meridiemMatch != null) {
            val rawHour = meridiemMatch.groups[1]?.value?.toIntOrNull() ?: return RequirementParsingResult.NotMatched
            val minute = meridiemMatch.groups[2]?.value?.toIntOrNull() ?: 0
            val meridiem = meridiemMatch.groups[3]?.value?.lowercase(Locale.ROOT) ?: return RequirementParsingResult.NotMatched
            val hour = when {
                meridiem == "pm" && rawHour < 12 -> rawHour + 12
                meridiem == "am" && rawHour == 12 -> 0
                else -> rawHour
            }
            val parsed = runCatching { LocalTime.of(hour, minute) }.getOrNull()
                ?: return RequirementParsingResult.Invalid("Please provide a valid reservation time.")
            if (parsed.isBefore(context.earliestReservationTime) || parsed.isAfter(context.latestReservationTime)) {
                return RequirementParsingResult.Invalid(
                    "Please choose a reservation time between ${formatTime(context.earliestReservationTime)} and ${formatTime(context.latestReservationTime)}."
                )
            }
            return RequirementParsingResult.Valid(TimeRequirementValue(raw = raw, value = parsed, displayValue = formatTime(parsed)))
        }

        val twentyFourHourMatch = TWENTY_FOUR_HOUR_PATTERN.find(raw) ?: return RequirementParsingResult.NotMatched
        val hour = twentyFourHourMatch.groups[1]?.value?.toIntOrNull() ?: return RequirementParsingResult.NotMatched
        val minute = twentyFourHourMatch.groups[2]?.value?.toIntOrNull() ?: 0
        val parsed = runCatching { LocalTime.of(hour, minute) }.getOrNull()
            ?: return RequirementParsingResult.Invalid("Please provide a valid reservation time.")
        if (parsed.isBefore(context.earliestReservationTime) || parsed.isAfter(context.latestReservationTime)) {
            return RequirementParsingResult.Invalid(
                "Please choose a reservation time between ${formatTime(context.earliestReservationTime)} and ${formatTime(context.latestReservationTime)}."
            )
        }
        return RequirementParsingResult.Valid(TimeRequirementValue(raw = raw, value = parsed, displayValue = formatTime(parsed)))
    }

    private fun formatTime(time: LocalTime): String =
        if (time.minute == 0) "%d:%02d".format(time.hour, time.minute) else "%d:%02d".format(time.hour, time.minute)

    private val MERIDIEM_TIME_PATTERN = Regex("""(?i)\b(\d{1,2})(?::(\d{2}))?\s*(am|pm)\b""")
    private val TWENTY_FOUR_HOUR_PATTERN = Regex("""(?i)\b([01]?\d|2[0-3])(?:h|:)([0-5]\d)?\b""")
}