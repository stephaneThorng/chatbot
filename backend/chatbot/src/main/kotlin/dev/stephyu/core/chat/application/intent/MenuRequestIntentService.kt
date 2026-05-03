package dev.stephyu.core.chat.application.intent

import dev.stephyu.core.chat.application.port.out.RestaurantKnowledgeRepository
import dev.stephyu.core.chat.application.state.ConversationStateInput
import dev.stephyu.core.chat.application.state.ConversationStateResult
import dev.stephyu.core.chat.domain.IntentName
import dev.stephyu.core.chat.domain.SlotName

class MenuRequestIntentService(
    private val knowledge: RestaurantKnowledgeRepository,
) : IntentService {
    override val intent: IntentName = IntentName.MENU_REQUEST
    override val policy: IntentPolicy = IntentPolicy(
        category = IntentCategory.INFORMATIONAL,
        clarifiable = true,
        supportsTopicContinuation = true,
        allowDuringWorkflow = true,
        entitySupport = setOf(SlotName.MENU_ITEM),
        disambiguationLabels = listOf("menu", "dishes", "food"),
    )

    override fun process(input: ConversationStateInput): ConversationStateResult {
        val normalized = input.message.lowercase()
        val requestedMenuItem = input.analysis.entities
            .firstOrNull { it.type == SlotName.MENU_ITEM }
            ?.value
            ?.lowercase()
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

        return ConversationStateResult(
            session = input.session.withInformationalIntent(intent),
            reply = "Menu highlights: " + matchingItems.joinToString("; ") {
                "${it.name} (${it.category}) - ${it.description} - ${it.price}"
            },
        )
    }
}
