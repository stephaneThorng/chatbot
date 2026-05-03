package dev.stephyu.core.chat.application.intent

import dev.stephyu.core.chat.application.port.out.RestaurantKnowledgeRepository
import dev.stephyu.core.chat.application.state.ConversationStateInput
import dev.stephyu.core.chat.application.state.ConversationStateResult
import dev.stephyu.core.chat.domain.IntentName
import dev.stephyu.core.chat.domain.SlotName

class ContactRequestIntentService(
    private val knowledge: RestaurantKnowledgeRepository,
) : IntentService {
    override val intent: IntentName = IntentName.CONTACT_REQUEST
    override val policy: IntentPolicy = IntentPolicy(
        category = IntentCategory.INFORMATIONAL,
        clarifiable = true,
        supportsTopicContinuation = true,
        allowDuringWorkflow = true,
        entitySupport = setOf(SlotName.PHONE, SlotName.EMAIL),
        disambiguationLabels = listOf("contact", "phone", "email"),
    )

    override fun process(input: ConversationStateInput): ConversationStateResult {
        val profile = knowledge.profile()
        return ConversationStateResult(
            session = input.session.withInformationalIntent(intent),
            reply = "You can contact ${profile.name} by phone at ${profile.phone} or by email at ${profile.email}.",
        )
    }
}
