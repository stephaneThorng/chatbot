package dev.stephyu.core.chat.application.state

import dev.stephyu.core.chat.application.service.ReplyComposer
import dev.stephyu.core.chat.application.service.ReservationWorkflowService

class WorkflowStateHandler(
    reservationWorkflowService: ReservationWorkflowService,
    replies: ReplyComposer,
) : BaseChatStateHandler(reservationWorkflowService, replies) {
    override fun process(input: ChatStateInput): ChatStateTransition =
        handleCommonIntent(input) ?: handleReservationWorkflow(input)
}
