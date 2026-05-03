package dev.stephyu.core.chat.application.intent.handler.knowledge

import dev.stephyu.core.chat.application.port.out.RestaurantKnowledgeRepository
import dev.stephyu.core.chat.application.state.ConversationTurnContext
import dev.stephyu.core.chat.application.state.ConversationTurnResult
import dev.stephyu.core.chat.application.intent.handler.IntentHandler
import dev.stephyu.core.chat.application.intent.policy.IntentCategory
import dev.stephyu.core.chat.application.intent.policy.IntentPolicy
import dev.stephyu.core.chat.domain.intent.IntentName
import dev.stephyu.core.chat.domain.nlp.SlotName

class PricingRequestIntentHandler(
    private val knowledge: RestaurantKnowledgeRepository,
) : IntentHandler {
    override val intent: IntentName = IntentName.PRICING_REQUEST
    override val policy: IntentPolicy = IntentPolicy(
        category = IntentCategory.INFORMATIONAL,
        clarifiable = true,
        supportsTopicContinuation = true,
        allowDuringWorkflow = true,
        entitySupport = setOf(SlotName.PRICE_ITEM, SlotName.MENU_ITEM),
        disambiguationLabels = listOf("price", "pricing", "cost"),
    )

    override fun process(input: ConversationTurnContext): ConversationTurnResult {
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

        return ConversationTurnResult(
            session = input.session.withInformationalIntent(intent),
            reply = reply,
        )
    }
}


