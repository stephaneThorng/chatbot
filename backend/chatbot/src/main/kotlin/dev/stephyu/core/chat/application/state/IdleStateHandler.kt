package dev.stephyu.core.chat.application.state

import dev.stephyu.core.chat.application.service.ReplyComposer
import dev.stephyu.core.chat.application.service.ReservationWorkflowService
import dev.stephyu.core.chat.application.service.isReservationWorkflowIntent

class IdleStateHandler(
    reservationWorkflowService: ReservationWorkflowService,
    replies: ReplyComposer,
) : BaseChatStateHandler(reservationWorkflowService, replies) {
    override fun process(input: ChatStateInput): ChatStateTransition =
        handleCommonIntent(input)
            ?: if (input.intent.isReservationWorkflowIntent()) handleReservationWorkflow(input) else unknown(input)
}
