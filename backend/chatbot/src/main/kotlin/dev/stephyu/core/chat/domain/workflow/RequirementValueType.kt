package dev.stephyu.core.chat.domain.workflow

import dev.stephyu.core.chat.domain.nlp.SlotName
import java.time.DayOfWeek
import java.time.LocalDate
import java.time.LocalTime
import java.time.Month
import java.time.format.TextStyle
import java.util.Locale

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

sealed interface RequirementValueType {
    val acceptedEntities: Set<SlotName>

    fun parse(raw: String, context: RequirementParsingContext): RequirementParsingResult
}

data class PersonNameRequirementType(
    private val minLength: Int = 2,
    private val maxLength: Int = 60,
) : RequirementValueType {
    override val acceptedEntities: Set<SlotName> = setOf(SlotName.NAME)

    override fun parse(raw: String, context: RequirementParsingContext): RequirementParsingResult {
        val candidate = extractName(raw).trim()
        if (candidate.isBlank()) return RequirementParsingResult.NotMatched
        if (candidate.length !in minLength..maxLength) {
            return RequirementParsingResult.Invalid("Please provide a name between $minLength and $maxLength characters.")
        }
        if (candidate.split(Regex("""\s+""")).size > 3) {
            return RequirementParsingResult.NotMatched
        }
        if (Regex("""(?i)\b(reservation|book|booking|cancel|modify|change|opening|hours|menu|price|pricing|contact|phone|email|location|address|need|want|would|like|make|new|please)\b""").containsMatchIn(candidate)) {
            return RequirementParsingResult.NotMatched
        }
        if (!NAME_PATTERN.matches(candidate)) {
            return RequirementParsingResult.Invalid("Please provide a valid reservation name.")
        }
        return RequirementParsingResult.Valid(TextRequirementValue(raw = raw, displayValue = candidate))
    }

    private fun extractName(raw: String): String {
        val trimmed = raw.trim().trimEnd('.', ',', ';')
        NAME_HINT_PATTERN.find(trimmed)?.let { match ->
            return match.groups[1]?.value?.trim()?.trimEnd('.', ',', ';') ?: trimmed
        }
        return trimmed
    }

    companion object {
        private val NAME_PATTERN = Regex("""[A-Za-zÀ-ÖØ-öø-ÿ][A-Za-zÀ-ÖØ-öø-ÿ'\- ]+""")
        private val NAME_HINT_PATTERN = Regex("""(?i)\b(?:under|for|name is|my name is|au nom de|nom de)\s+([A-Za-zÀ-ÖØ-öø-ÿ][A-Za-zÀ-ÖØ-öø-ÿ'\- ]{1,60})\b""")
    }
}

data object DateRequirementType : RequirementValueType {
    override val acceptedEntities: Set<SlotName> = setOf(SlotName.DATE)

    override fun parse(raw: String, context: RequirementParsingContext): RequirementParsingResult {
        val normalized = raw.lowercase(Locale.ROOT)
        val parsed = parseRelativeDate(normalized, context.today)
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

data object PartySizeRequirementType : RequirementValueType {
    override val acceptedEntities: Set<SlotName> = setOf(SlotName.PEOPLE)

    override fun parse(raw: String, context: RequirementParsingContext): RequirementParsingResult {
        val candidate = PEOPLE_PATTERN.find(raw)?.groups?.drop(1)?.firstNotNullOfOrNull { it?.value }
            ?: raw.trim().takeIf { PEOPLE_ONLY_PATTERN.matches(it) }
            ?: return RequirementParsingResult.NotMatched
        val value = candidate.toIntOrNull() ?: return RequirementParsingResult.NotMatched
        if (value !in context.minPartySize..context.maxPartySize) {
            return RequirementParsingResult.Invalid(
                "We can accept parties from ${context.minPartySize} to ${context.maxPartySize} people. For how many people should I book?"
            )
        }
        return RequirementParsingResult.Valid(IntegerRequirementValue(raw = raw, value = value, displayValue = value.toString()))
    }

    private val PEOPLE_PATTERN = Regex("""(?i)\b(?:for\s*)?(\d{1,3})\s*(?:people|persons|guests|personnes|couverts)\b|\bfor\s*(\d{1,3})\b""")
    private val PEOPLE_ONLY_PATTERN = Regex("""\s*\d{1,3}\s*""")
}

data object ConfirmationRequirementType : RequirementValueType {
    override val acceptedEntities: Set<SlotName> = emptySet()

    override fun parse(raw: String, context: RequirementParsingContext): RequirementParsingResult {
        val normalized = raw.trim().lowercase(Locale.ROOT)
        if (isYes(normalized)) {
            return RequirementParsingResult.Valid(BooleanRequirementValue(raw = raw, value = true, displayValue = "yes"))
        }
        if (isNo(normalized)) {
            return RequirementParsingResult.Valid(BooleanRequirementValue(raw = raw, value = false, displayValue = "no"))
        }
        return RequirementParsingResult.NotMatched
    }

    private fun isYes(normalized: String): Boolean =
        normalized in setOf("yes", "y", "ok", "okay", "confirm", "confirmed", "sure", "oui") ||
            normalized.startsWith("yes ") ||
            "confirm it" in normalized ||
            "you can confirm" in normalized

    private fun isNo(normalized: String): Boolean =
        normalized in setOf("no", "n", "nope", "change", "non") ||
            normalized.startsWith("no ")
}


