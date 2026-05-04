package dev.stephyu.core.chat.application.intent.handler.reservation

import dev.stephyu.core.chat.application.state.StateHandlerInput
import dev.stephyu.core.chat.application.state.StateHandlerResult
import dev.stephyu.core.chat.application.intent.handler.IntentHandler
import dev.stephyu.core.chat.application.intent.policy.IntentCategory
import dev.stephyu.core.chat.application.intent.policy.IntentPolicy
import dev.stephyu.core.chat.domain.intent.IntentName
import dev.stephyu.core.chat.domain.nlp.SlotName

class ReservationStatusIntentHandler : IntentHandler {
    override val intent: IntentName = IntentName.RESERVATION_STATUS
    override val policy: IntentPolicy = IntentPolicy(category = IntentCategory.STATUS)

    override fun process(input: StateHandlerInput): StateHandlerResult {
        val reservation = input.session.completedWorkflows[IntentName.RESERVATION_MODIFY]
            ?: input.session.completedWorkflows[IntentName.RESERVATION_CREATE]?:
            return StateHandlerResult(
                updatedSession = input.session.withoutWorkflow(nextIntent = intent),
                reply = "I do not have a confirmed reservation in this session yet.",
            )

        return StateHandlerResult(
            updatedSession = input.session.withoutWorkflow(nextIntent = intent),
            reply = "Your reservation is confirmed: ${summary(reservation.filledSlots())}.",
            slotSnapshot = reservation.filledSlots(),
        )
    }


    private fun summary(slots: Map<SlotName, String>): String {
        val parts = buildList {
            slots[SlotName.PEOPLE]?.let { add("$it people") }
            slots[SlotName.DATE]?.let { add("on $it") }
            slots[SlotName.TIME]?.let { add("at $it") }
            slots[SlotName.NAME]?.let { add("under $it") } // Ajout direct, sans condition
        }.joinToString(" ")

        return parts.ifBlank { "no details captured" }
    }
}


