package dev.stephyu.core.chat.application.intent.handler.knowledge

import dev.stephyu.core.chat.application.port.out.RestaurantKnowledgeRepository
import dev.stephyu.core.chat.application.state.ConversationTurnContext
import dev.stephyu.core.chat.application.state.ConversationTurnResult
import dev.stephyu.core.chat.application.intent.handler.IntentHandler
import dev.stephyu.core.chat.application.intent.policy.IntentCategory
import dev.stephyu.core.chat.application.intent.policy.IntentPolicy
import dev.stephyu.core.chat.domain.intent.IntentName
import dev.stephyu.core.chat.domain.knowledge.OpeningHour
import dev.stephyu.core.chat.domain.nlp.SlotName
import java.time.LocalTime

class OpeningHoursIntentHandler(
    private val knowledge: RestaurantKnowledgeRepository,
) : IntentHandler {
    override val intent: IntentName = IntentName.OPENING_HOURS
    override val policy: IntentPolicy = IntentPolicy(
        category = IntentCategory.INFORMATIONAL,
        supportsTopicContinuation = true,
        allowDuringWorkflow = true,
        entitySupport = setOf(SlotName.DATE, SlotName.TIME),
        disambiguationLabels = listOf("hours", "opening hours", "open"),
    )

    override fun process(input: ConversationTurnContext): ConversationTurnResult {
        val hours = knowledge.openingHours()
        val requestedDay = requestedOpeningDay(input, hours)
        val reply = if (requestedDay != null) {
            if (requestedDay.opensAt == null || requestedDay.closesAt == null) {
                "On ${requestedDay.day}, we are closed."
            } else {
                val requestedTime = requestedOpeningTime(input)
                val base = "On ${requestedDay.day}, we are open from ${requestedDay.opensAt} to ${requestedDay.closesAt}."
                if (requestedTime == null) {
                    base
                } else {
                    val opensAt = LocalTime.parse(requestedDay.opensAt)
                    val closesAt = LocalTime.parse(requestedDay.closesAt)
                    val available = !requestedTime.isBefore(opensAt) && requestedTime.isBefore(closesAt)
                    val availability = if (available) "we should be open" else "we are closed"
                    "$base At ${formatTime(requestedTime)}, $availability."
                }
            }
        } else {
            val formattedHours = hours.joinToString("; ") { hour ->
                if (hour.opensAt == null || hour.closesAt == null) "${hour.day}: closed"
                else "${hour.day}: ${hour.opensAt}-${hour.closesAt}"
            }
            "Our opening hours are $formattedHours."
        }

        return ConversationTurnResult(
            session = input.session.withInformationalIntent(intent),
            reply = reply,
        )
    }

    private fun requestedOpeningDay(input: ConversationTurnContext, hours: List<OpeningHour>): OpeningHour? {
        val normalized = input.message.lowercase()
        val dateEntities = input.analysis.entities
            .filter { it.type == SlotName.DATE }
            .map { it.value.lowercase() }
        return hours.firstOrNull { hour ->
            val day = hour.day.lowercase()
            dateEntities.any { entity -> Regex("""\b${Regex.escape(day)}\b""").containsMatchIn(entity) } ||
                Regex("""\b${Regex.escape(day)}\b""").containsMatchIn(normalized)
        }
    }

    private fun requestedOpeningTime(input: ConversationTurnContext): LocalTime? {
        val candidates = input.analysis.entities.filter { it.type == SlotName.TIME }.map { it.value } + input.message
        candidates.forEach { candidate ->
            parseTime(candidate)?.let { return it }
        }
        return null
    }

    private fun parseTime(text: String): LocalTime? {
        val meridiemMatch = MERIDIEM_TIME_PATTERN.find(text)
        if (meridiemMatch != null) {
            val rawHour = meridiemMatch.groups[1]?.value?.toIntOrNull() ?: return null
            val minute = meridiemMatch.groups[2]?.value?.toIntOrNull() ?: 0
            val meridiem = meridiemMatch.groups[3]?.value?.lowercase() ?: return null
            val hour = when {
                meridiem == "pm" && rawHour < 12 -> rawHour + 12
                meridiem == "am" && rawHour == 12 -> 0
                else -> rawHour
            }
            return runCatching { LocalTime.of(hour, minute) }.getOrNull()
        }

        val twentyFourHourMatch = TWENTY_FOUR_HOUR_PATTERN.find(text) ?: return null
        val hour = twentyFourHourMatch.groups[1]?.value?.toIntOrNull() ?: return null
        val minute = twentyFourHourMatch.groups[2]?.value?.toIntOrNull() ?: return null
        return runCatching { LocalTime.of(hour, minute) }.getOrNull()
    }

    private fun formatTime(time: LocalTime): String =
        "%02d:%02d".format(time.hour, time.minute)

    companion object {
        private val MERIDIEM_TIME_PATTERN = Regex("""(?i)\b(\d{1,2})(?::(\d{2}))?\s*(am|pm)\b""")
        private val TWENTY_FOUR_HOUR_PATTERN = Regex("""\b([01]?\d|2[0-3]):([0-5]\d)\b""")
    }
}


