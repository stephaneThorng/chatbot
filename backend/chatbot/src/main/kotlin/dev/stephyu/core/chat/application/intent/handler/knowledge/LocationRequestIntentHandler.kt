package dev.stephyu.core.chat.application.intent.handler.knowledge

import dev.stephyu.core.chat.application.port.out.RestaurantKnowledgeRepository
import dev.stephyu.core.chat.application.state.ConversationTurnContext
import dev.stephyu.core.chat.application.state.ConversationTurnResult
import dev.stephyu.core.chat.application.intent.handler.IntentHandler
import dev.stephyu.core.chat.application.intent.policy.IntentCategory
import dev.stephyu.core.chat.application.intent.policy.IntentPolicy
import dev.stephyu.core.chat.domain.intent.IntentName
import dev.stephyu.core.chat.domain.nlp.SlotName

class LocationRequestIntentHandler(
    private val knowledge: RestaurantKnowledgeRepository,
) : IntentHandler {
    override val intent: IntentName = IntentName.LOCATION_REQUEST
    override val policy: IntentPolicy = IntentPolicy(
        category = IntentCategory.INFORMATIONAL,
        clarifiable = true,
        supportsTopicContinuation = true,
        allowDuringWorkflow = true,
        entitySupport = setOf(SlotName.LOCATION),
        disambiguationLabels = listOf("location", "address", "parking"),
    )

    override fun process(input: ConversationTurnContext): ConversationTurnResult {
        val profile = knowledge.profile()
        return ConversationTurnResult(
            session = input.session.withInformationalIntent(intent),
            reply = "${profile.name} is located at ${profile.address}. ${profile.parkingHints.joinToString(" ")}",
        )
    }
}


