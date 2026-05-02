package dev.stephyu.core.chat.application.service

import dev.stephyu.core.chat.application.port.out.RestaurantKnowledgeRepository
import dev.stephyu.core.chat.domain.ConversationAct
import dev.stephyu.core.chat.domain.EntityType
import dev.stephyu.core.chat.domain.IntentName
import dev.stephyu.core.chat.domain.NlpAnalysis
import dev.stephyu.core.chat.domain.NlpEntity
import dev.stephyu.core.chat.domain.OpeningHour
import dev.stephyu.core.chat.domain.SlotName
import dev.stephyu.core.chat.domain.workflow.WorkflowInstance
import java.time.LocalTime

class ReplyComposer(
    private val knowledge: RestaurantKnowledgeRepository,
) {
    fun conversationActReply(conversationAct: ConversationAct): String = when (conversationAct) {
        ConversationAct.GREETING -> "Hello. How can I help you today?"
        ConversationAct.THANKS -> "You're welcome."
        ConversationAct.FAREWELL -> "Goodbye. See you soon."
    }

    fun informationalReply(intent: IntentName, context: ReplyContext): String = when (intent) {
        IntentName.MENU_REQUEST -> menuReply(context)
        IntentName.OPENING_HOURS -> openingHoursReply(context)
        IntentName.LOCATION_REQUEST -> locationReply(context)
        IntentName.PRICING_REQUEST -> pricingReply(context)
        IntentName.CONTACT_REQUEST -> contactReply(context)
        else -> "I can help with restaurant information."
    }

    fun unknownReply(): String =
        "I can help with reservations, opening hours, location, menu, prices, and contact details."

    fun emptyMessageReply(): String =
        "Please send a message so I can help you."

    fun applyConversationActPrefix(preprocessed: PreprocessedMessage, reply: String): String =
        if (preprocessed.hasLeadingGreeting && !reply.startsWith("Hello.")) "Hello. $reply" else reply

    fun resumeReservationPrompt(workflow: WorkflowInstance): String {
        val prompt = workflow.firstMissingRequirement()?.prompt?.defaultText
            ?: confirmationPrompt(workflow.wireSlots())
        return " We can continue your reservation when you are ready: $prompt"
    }

    fun confirmationPrompt(slots: Map<SlotName, String>): String =
        "I have a reservation for ${slots[SlotName.PEOPLE]} people on ${slots[SlotName.DATE]} at ${slots[SlotName.TIME]}, under ${slots[SlotName.NAME]}. Should I confirm it?"

    fun reservationSummary(slots: Map<SlotName, String>): String =
        listOfNotNull(
            slots[SlotName.PEOPLE]?.let { "$it people" },
            slots[SlotName.DATE]?.let { "on $it" },
            slots[SlotName.TIME]?.let { "at $it" },
        ).joinToString(" ")
            .let { summary ->
                slots[SlotName.NAME]?.let { name ->
                    if (summary.isBlank()) "under $name" else "$summary, under $name"
                } ?: summary
            }
            .ifBlank { "no details captured" }

    private fun openingHoursReply(context: ReplyContext): String {
        val hours = knowledge.openingHours()
        val requestedDay = requestedOpeningDay(context, hours)
        if (requestedDay != null) {
            if (requestedDay.opensAt == null || requestedDay.closesAt == null) {
                return "On ${requestedDay.day}, we are closed."
            }
            val requestedTime = requestedOpeningTime(context)
            val base = "On ${requestedDay.day}, we are open from ${requestedDay.opensAt} to ${requestedDay.closesAt}."
            if (requestedTime == null) {
                return base
            }
            val opensAt = LocalTime.parse(requestedDay.opensAt)
            val closesAt = LocalTime.parse(requestedDay.closesAt)
            val available = !requestedTime.isBefore(opensAt) && requestedTime.isBefore(closesAt)
            val availability = if (available) "we should be open" else "we are closed"
            return "$base At ${formatTime(requestedTime)}, $availability."
        }
        val formattedHours = hours.joinToString("; ") { hour ->
            if (hour.opensAt == null || hour.closesAt == null) "${hour.day}: closed"
            else "${hour.day}: ${hour.opensAt}-${hour.closesAt}"
        }
        return "Our opening hours are $formattedHours."
    }

    private fun locationReply(context: ReplyContext): String {
        val profile = knowledge.profile()
        return "${profile.name} is located at ${profile.address}. ${profile.parkingHints.joinToString(" ")}"
    }

    private fun contactReply(context: ReplyContext): String {
        val profile = knowledge.profile()
        return "You can contact ${profile.name} by phone at ${profile.phone} or by email at ${profile.email}."
    }

    private fun menuReply(context: ReplyContext): String {
        val normalized = context.message.lowercase()
        val requestedMenuItem = context.firstEntityValue(EntityType.MENU_ITEM)?.lowercase()
        val items = knowledge.menuItems()
        val matchingItems = items.filter { item ->
            val normalizedName = item.name.lowercase()
            val normalizedCategory = item.category.lowercase()
            normalizedCategory in normalized ||
                (item.category.lowercase() == "dessert" && "desert" in normalized) ||
                item.tags.any { it.lowercase() in normalized } ||
                normalizedName in normalized ||
                requestedMenuItem?.let { entity ->
                    normalizedCategory in entity ||
                        item.tags.any { it.lowercase() in entity } ||
                        normalizedName in entity ||
                        entity in normalizedName
                } == true
        }.ifEmpty { items.take(5) }
        return "Menu highlights: " + matchingItems.joinToString("; ") {
            "${it.name} (${it.category}) - ${it.description} - ${it.price}"
        }
    }

    private fun pricingReply(context: ReplyContext): String {
        val normalized = context.message.lowercase()
        val requestedPriceItem = context.firstEntityValue(EntityType.PRICE_ITEM)?.lowercase()
        val itemMatch = knowledge.menuItems().firstOrNull { item ->
            val normalizedName = item.name.lowercase()
            normalizedName in normalized ||
                requestedPriceItem?.let { entity -> normalizedName in entity || entity in normalizedName } == true
        }
        if (itemMatch != null) {
            return "${itemMatch.name} costs ${itemMatch.price}."
        }
        return "Price guide: " + knowledge.priceInfo().joinToString("; ") { "${it.label}: ${it.value}" }
    }

    private fun requestedOpeningDay(context: ReplyContext, hours: List<OpeningHour>): OpeningHour? {
        val normalized = context.message.lowercase()
        val dateEntities = context.entityValues(EntityType.DATE).map { it.lowercase() }
        return hours.firstOrNull { hour ->
            val day = hour.day.lowercase()
            dateEntities.any { entity -> Regex("""\b${Regex.escape(day)}\b""").containsMatchIn(entity) } ||
                Regex("""\b${Regex.escape(day)}\b""").containsMatchIn(normalized)
        }
    }

    private fun requestedOpeningTime(context: ReplyContext): LocalTime? {
        val candidateTexts = context.entityValues(EntityType.TIME) + context.message
        candidateTexts.forEach { candidate ->
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

    private fun ReplyContext.firstEntityValue(type: EntityType): String? =
        entityValues(type).firstOrNull()

    private fun ReplyContext.entityValues(type: EntityType): List<String> =
        analysis.entities
            .filter { it.type == type }
            .map(NlpEntity::value)

    private fun formatTime(time: LocalTime): String =
        "%02d:%02d".format(time.hour, time.minute)

    companion object {
        private val MERIDIEM_TIME_PATTERN = Regex("""(?i)\b(\d{1,2})(?::(\d{2}))?\s*(am|pm)\b""")
        private val TWENTY_FOUR_HOUR_PATTERN = Regex("""\b([01]?\d|2[0-3]):([0-5]\d)\b""")
    }
}

data class ReplyContext(
    val message: String,
    val analysis: NlpAnalysis,
)
