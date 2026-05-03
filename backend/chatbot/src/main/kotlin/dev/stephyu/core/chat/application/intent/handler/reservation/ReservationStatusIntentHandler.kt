package dev.stephyu.core.chat.application.intent.handler.reservation

import dev.stephyu.core.chat.application.state.ConversationTurnContext
import dev.stephyu.core.chat.application.state.ConversationTurnResult
import dev.stephyu.core.chat.application.intent.handler.IntentHandler
import dev.stephyu.core.chat.application.intent.policy.IntentCategory
import dev.stephyu.core.chat.application.intent.policy.IntentPolicy
import dev.stephyu.core.chat.domain.intent.IntentName
import dev.stephyu.core.chat.domain.nlp.SlotName

class ReservationStatusIntentHandler : IntentHandler {
    override val intent: IntentName = IntentName.RESERVATION_STATUS
    override val policy: IntentPolicy = IntentPolicy(category = IntentCategory.STATUS)

    override fun process(input: ConversationTurnContext): ConversationTurnResult {
        val reservation = input.session.completedWorkflows[IntentName.RESERVATION_MODIFY]
            ?: input.session.completedWorkflows[IntentName.RESERVATION_CREATE]?:
            return ConversationTurnResult(
                session = input.session.withoutWorkflow(nextIntent = intent),
                reply = "I do not have a confirmed reservation in this session yet.",
            )

        return ConversationTurnResult(
            session = input.session.withoutWorkflow(nextIntent = intent),
            reply = "Your reservation is confirmed: ${reservationSummary(reservation.filledSlots())}.",
            slots = reservation.filledSlots(),
        )
    }

    private fun reservationSummary(slots: Map<SlotName, String>): String =
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
}


