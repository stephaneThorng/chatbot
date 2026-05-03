package dev.stephyu.core.chat.application.intent

import dev.stephyu.core.chat.application.state.ConversationStateInput
import dev.stephyu.core.chat.application.state.ConversationStateResult
import dev.stephyu.core.chat.domain.IntentName
import dev.stephyu.core.chat.domain.SlotName

class ReservationStatusIntentService : IntentService {
    override val intent: IntentName = IntentName.RESERVATION_STATUS
    override val policy: IntentPolicy = IntentPolicy(category = IntentCategory.STATUS)

    override fun process(input: ConversationStateInput): ConversationStateResult {
        val reservation = input.session.completedWorkflows[IntentName.RESERVATION_MODIFY]
            ?: input.session.completedWorkflows[IntentName.RESERVATION_CREATE]?:
            return ConversationStateResult(
                session = input.session.withoutWorkflow(nextIntent = intent),
                reply = "I do not have a confirmed reservation in this session yet.",
            )

        return ConversationStateResult(
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
