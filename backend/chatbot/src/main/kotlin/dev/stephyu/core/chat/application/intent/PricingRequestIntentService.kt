package dev.stephyu.core.chat.application.intent

import dev.stephyu.core.chat.application.port.out.RestaurantKnowledgeRepository
import dev.stephyu.core.chat.application.state.ConversationStateInput
import dev.stephyu.core.chat.application.state.ConversationStateResult
import dev.stephyu.core.chat.domain.IntentName
import dev.stephyu.core.chat.domain.SlotName

class PricingRequestIntentService(
    private val knowledge: RestaurantKnowledgeRepository,
) : IntentService {
    override val intent: IntentName = IntentName.PRICING_REQUEST
    override val policy: IntentPolicy = IntentPolicy(
        category = IntentCategory.INFORMATIONAL,
        clarifiable = true,
        supportsTopicContinuation = true,
        allowDuringWorkflow = true,
        entitySupport = setOf(SlotName.PRICE_ITEM, SlotName.MENU_ITEM),
        disambiguationLabels = listOf("price", "pricing", "cost"),
    )

    override fun process(input: ConversationStateInput): ConversationStateResult {
        val normalized = input.message.lowercase()
        val requestedPriceItem = input.analysis.entities
            .firstOrNull { it.type == SlotName.PRICE_ITEM }
            ?.value
            ?.lowercase()
        val itemMatch = knowledge.menuItems().firstOrNull { item ->
            val normalizedName = item.name.lowercase()
            normalizedName in normalized ||
                requestedPriceItem?.let { entity -> normalizedName in entity || entity in normalizedName } == true
        }
        val reply = if (itemMatch != null) {
            "${itemMatch.name} costs ${itemMatch.price}."
        } else {
            "Price guide: " + knowledge.priceInfo().joinToString("; ") { "${it.label}: ${it.value}" }
        }

        return ConversationStateResult(
            session = input.session.withInformationalIntent(intent),
            reply = reply,
        )
    }
}
