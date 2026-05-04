package dev.stephyu.core.chat.domain.workflow.requirement.valuetype

import dev.stephyu.core.chat.domain.nlp.SlotName
import dev.stephyu.core.chat.domain.workflow.DateRequirementValue
import dev.stephyu.core.chat.domain.workflow.RequirementParsingContext
import dev.stephyu.core.chat.domain.workflow.RequirementParsingResult
import dev.stephyu.core.chat.domain.workflow.RequirementValueType
import java.time.DayOfWeek
import java.time.LocalDate
import java.time.Month
import java.time.format.TextStyle
import java.util.Locale

data object DateRequirementType : RequirementValueType {
    override val acceptedEntities: Set<SlotName> = setOf(SlotName.DATE)

    override fun parse(raw: String, context: RequirementParsingContext): RequirementParsingResult {
        val normalized = raw.lowercase(Locale.ROOT)
        val parsed = parseIsoDate(normalized)
            ?: parseRelativeDate(normalized, context.today)
            ?: parseWeekday(normalized, context.today)
            ?: parseMonthDay(normalized, context.today)
            ?: return RequirementParsingResult.NotMatched

        if (parsed.isBefore(context.today)) {
            return RequirementParsingResult.Invalid("Please provide a future reservation date.")
        }

        return RequirementParsingResult.Valid(
            DateRequirementValue(
                raw = raw,
                value = parsed,
                displayValue = formatDate(parsed),
            )
        )
    }

    private fun parseIsoDate(text: String): LocalDate? =
        runCatching { LocalDate.parse(text.trim()) }.getOrNull()

    private fun parseRelativeDate(text: String, today: LocalDate): LocalDate? = when {
        Regex("""\b(today|tonight|ce soir|aujourd'hui)\b""").containsMatchIn(text) -> today
        Regex("""\b(tomorrow|demain)\b""").containsMatchIn(text) -> today.plusDays(1)
        Regex("""\b(yesterday|hier)\b""").containsMatchIn(text) -> today.minusDays(1)
        else -> null
    }

    private fun parseWeekday(text: String, today: LocalDate): LocalDate? {
        val weekday = WEEKDAYS.entries.firstOrNull { (name, _) ->
            Regex("""\b${Regex.escape(name)}\b""").containsMatchIn(text)
        }?.value ?: return null
        var candidate = today
        do {
            candidate = candidate.plusDays(1)
        } while (candidate.dayOfWeek != weekday)
        return candidate
    }

    private fun parseMonthDay(text: String, today: LocalDate): LocalDate? {
        MONTHS.entries.forEach { (monthName, month) ->
            Regex("""\b${Regex.escape(monthName)}\s+(\d{1,2})\b""").find(text)?.let { match ->
                return buildMonthDay(today, month, match.groups[1]?.value?.toIntOrNull())
            }
            Regex("""\b(\d{1,2})\s+${Regex.escape(monthName)}\b""").find(text)?.let { match ->
                return buildMonthDay(today, month, match.groups[1]?.value?.toIntOrNull())
            }
        }
        return null
    }

    private fun buildMonthDay(today: LocalDate, month: Month, day: Int?): LocalDate? {
        if (day == null) return null
        val thisYear = runCatching { LocalDate.of(today.year, month, day) }.getOrNull() ?: return null
        return if (thisYear.isBefore(today)) thisYear.plusYears(1) else thisYear
    }

    private fun formatDate(date: LocalDate): String {
        val month = date.month.getDisplayName(TextStyle.FULL, Locale.ENGLISH)
        return "$month ${date.dayOfMonth}, ${date.year}"
    }

    private val WEEKDAYS = mapOf(
        "monday" to DayOfWeek.MONDAY,
        "tuesday" to DayOfWeek.TUESDAY,
        "wednesday" to DayOfWeek.WEDNESDAY,
        "thursday" to DayOfWeek.THURSDAY,
        "friday" to DayOfWeek.FRIDAY,
        "saturday" to DayOfWeek.SATURDAY,
        "sunday" to DayOfWeek.SUNDAY,
        "lundi" to DayOfWeek.MONDAY,
        "mardi" to DayOfWeek.TUESDAY,
        "mercredi" to DayOfWeek.WEDNESDAY,
        "jeudi" to DayOfWeek.THURSDAY,
        "vendredi" to DayOfWeek.FRIDAY,
        "samedi" to DayOfWeek.SATURDAY,
        "dimanche" to DayOfWeek.SUNDAY,
    )

    private val MONTHS = mapOf(
        "january" to Month.JANUARY,
        "february" to Month.FEBRUARY,
        "march" to Month.MARCH,
        "april" to Month.APRIL,
        "may" to Month.MAY,
        "june" to Month.JUNE,
        "july" to Month.JULY,
        "august" to Month.AUGUST,
        "september" to Month.SEPTEMBER,
        "october" to Month.OCTOBER,
        "november" to Month.NOVEMBER,
        "december" to Month.DECEMBER,
        "janvier" to Month.JANUARY,
        "février" to Month.FEBRUARY,
        "fevrier" to Month.FEBRUARY,
        "mars" to Month.MARCH,
        "avril" to Month.APRIL,
        "mai" to Month.MAY,
        "juin" to Month.JUNE,
        "juillet" to Month.JULY,
        "août" to Month.AUGUST,
        "aout" to Month.AUGUST,
        "septembre" to Month.SEPTEMBER,
        "octobre" to Month.OCTOBER,
        "novembre" to Month.NOVEMBER,
        "décembre" to Month.DECEMBER,
        "decembre" to Month.DECEMBER,
    )
}