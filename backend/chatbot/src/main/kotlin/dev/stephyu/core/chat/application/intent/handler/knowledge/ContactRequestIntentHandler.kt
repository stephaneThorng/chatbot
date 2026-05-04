package dev.stephyu.core.chat.application.intent.handler.knowledge

import dev.stephyu.core.chat.application.port.out.RestaurantKnowledgeRepository
import dev.stephyu.core.chat.application.state.StateHandlerInput
import dev.stephyu.core.chat.application.state.StateHandlerResult
import dev.stephyu.core.chat.application.intent.handler.IntentHandler
import dev.stephyu.core.chat.application.intent.policy.IntentCategory
import dev.stephyu.core.chat.application.intent.policy.IntentPolicy
import dev.stephyu.core.chat.domain.intent.IntentName
import dev.stephyu.core.chat.domain.nlp.SlotName

class ContactRequestIntentHandler(
    private val knowledge: RestaurantKnowledgeRepository,
) : IntentHandler {
    override val intent: IntentName = IntentName.CONTACT_REQUEST
    override val policy: IntentPolicy = IntentPolicy(
        category = IntentCategory.INFORMATIONAL,
        clarifiable = true,
        supportsTopicContinuation = true,
        allowDuringWorkflow = true,
        entitySupport = setOf(SlotName.PHONE, SlotName.EMAIL),
        disambiguationLabels = listOf("contact", "phone", "email"),
    )

    override fun process(input: StateHandlerInput): StateHandlerResult {
        val profile = knowledge.profile()
        return StateHandlerResult(
            updatedSession = input.session.withInformationalIntent(intent),
            reply = "You can contact ${profile.name} by phone at ${profile.phone} or by email at ${profile.email}.",
        )
    }
}


