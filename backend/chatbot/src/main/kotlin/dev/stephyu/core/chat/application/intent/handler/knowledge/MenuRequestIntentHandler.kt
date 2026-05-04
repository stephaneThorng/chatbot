package dev.stephyu.core.chat.application.intent.handler.knowledge

import dev.stephyu.core.chat.application.port.out.RestaurantKnowledgeRepository
import dev.stephyu.core.chat.application.state.StateHandlerInput
import dev.stephyu.core.chat.application.state.StateHandlerResult
import dev.stephyu.core.chat.application.intent.handler.IntentHandler
import dev.stephyu.core.chat.application.intent.policy.IntentCategory
import dev.stephyu.core.chat.application.intent.policy.IntentPolicy
import dev.stephyu.core.chat.domain.intent.IntentName
import dev.stephyu.core.chat.domain.nlp.SlotName

class MenuRequestIntentHandler(
    private val knowledge: RestaurantKnowledgeRepository,
) : IntentHandler {
    override val intent: IntentName = IntentName.MENU_REQUEST
    override val policy: IntentPolicy = IntentPolicy(
        category = IntentCategory.INFORMATIONAL,
        clarifiable = true,
        supportsTopicContinuation = true,
        allowDuringWorkflow = true,
        entitySupport = setOf(SlotName.MENU_ITEM),
        disambiguationLabels = listOf("menu", "dishes", "food"),
    )

    override fun process(input: StateHandlerInput): StateHandlerResult {
        val normalized = input.processedText.lowercase()
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

        return StateHandlerResult(
            updatedSession = input.session.withInformationalIntent(intent),
            reply = "Menu highlights: " + matchingItems.joinToString("; ") {
                "${it.name} (${it.category}) - ${it.description} - ${it.price}"
            },
        )
    }
}


