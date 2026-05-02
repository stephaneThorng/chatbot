package dev.stephyu.core.chat.application.state

import dev.stephyu.core.chat.application.service.ReplyComposer
import dev.stephyu.core.chat.application.service.ReplyContext
import dev.stephyu.core.chat.application.service.ReservationWorkflowService
import dev.stephyu.core.chat.application.service.isInformationalIntent
import dev.stephyu.core.chat.domain.ConversationState
import dev.stephyu.core.chat.domain.IntentName
import dev.stephyu.core.chat.domain.reservation.ReservationTransition
import dev.stephyu.core.chat.domain.workflow.WorkflowInput

abstract class BaseChatStateHandler(
    private val reservationWorkflowService: ReservationWorkflowService,
    private val replies: ReplyComposer,
) : ChatStateHandler {
    protected fun handleCommonIntent(input: ChatStateInput): ChatStateTransition? =
        when {
            input.intent == IntentName.RESERVATION_STATUS -> reservationWorkflowService.status(input.session).toChatTransition()
            input.intent.isInformationalIntent() -> handleInformationalIntent(input)
            else -> null
        }

    protected fun handleReservationWorkflow(input: ChatStateInput): ChatStateTransition =
        reservationWorkflowService.handle(
            WorkflowInput(
                session = input.session,
                intent = input.intent,
                message = input.message,
                analysis = input.analysis,
            )
        ).toChatTransition()

    protected fun unknown(input: ChatStateInput): ChatStateTransition =
        ChatStateTransition(
            session = input.session.copy(state = ConversationState.IDLE),
            reply = replies.unknownReply(),
        )

    private fun handleInformationalIntent(input: ChatStateInput): ChatStateTransition {
        val answer = replies.informationalReply(
            intent = input.intent,
            context = ReplyContext(
                message = input.message,
                analysis = input.analysis,
            ),
        )
        val currentWorkflow = input.session.currentWorkflow
        val suffix = if (currentWorkflow != null) {
            replies.resumeReservationPrompt(currentWorkflow)
        } else {
            ""
        }
        return ChatStateTransition(
            session = input.session.copy(
                previousIntent = input.session.currentIntent,
                currentIntent = input.intent,
            ),
            reply = answer + suffix,
        )
    }

    private fun ReservationTransition.toChatTransition(): ChatStateTransition =
        ChatStateTransition(
            session = session,
            reply = reply,
            slots = slots,
            missingSlots = missingSlots,
            completed = completed,
        )
}
