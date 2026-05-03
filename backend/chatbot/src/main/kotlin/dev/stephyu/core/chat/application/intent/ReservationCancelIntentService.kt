package dev.stephyu.core.chat.application.intent

import dev.stephyu.core.chat.application.state.ConversationStateInput
import dev.stephyu.core.chat.application.state.ConversationStateResult
import dev.stephyu.core.chat.application.workflow.WorkflowEngine
import dev.stephyu.core.chat.application.workflow.WorkflowEngineInput
import dev.stephyu.core.chat.application.workflow.WorkflowOutcome
import dev.stephyu.core.chat.domain.ConversationSession
import dev.stephyu.core.chat.domain.IntentName
import dev.stephyu.core.chat.domain.SlotName
import dev.stephyu.core.chat.domain.workflow.ConfirmationRequirementType
import dev.stephyu.core.chat.domain.workflow.RequirementActivation
import dev.stephyu.core.chat.domain.workflow.RequirementName
import dev.stephyu.core.chat.domain.workflow.RequirementPrompt
import dev.stephyu.core.chat.domain.workflow.WorkflowDefinition
import dev.stephyu.core.chat.domain.workflow.WorkflowRequirementDefinition

class ReservationCancelIntentService(
    private val workflowEngine: WorkflowEngine,
) : IntentService {
    override val intent: IntentName = IntentName.RESERVATION_CANCEL
    override val policy: IntentPolicy = IntentPolicy(category = IntentCategory.WORKFLOW)

    override fun workflowDefinition(session: ConversationSession): WorkflowDefinition? {
        session.completedWorkflows[IntentName.RESERVATION_MODIFY]
            ?: session.completedWorkflows[IntentName.RESERVATION_CREATE]?: return null

        return WorkflowDefinition(
            ownerIntent = intent,
            requirements = listOf(
                WorkflowRequirementDefinition(
                    name = RequirementName.CONFIRMATION,
                    valueType = ConfirmationRequirementType,
                    prompt = RequirementPrompt("ask_confirmation", "Please confirm with yes or no."),
                    activation = RequirementActivation.ALWAYS,
                )
            ),
            canCancel = false,
        )
    }

    override fun process(input: ConversationStateInput): ConversationStateResult {
        val reservation = input.session.completedWorkflows[IntentName.RESERVATION_MODIFY]
            ?: input.session.completedWorkflows[IntentName.RESERVATION_CREATE]
            ?: return ConversationStateResult(
                session = input.session.withoutWorkflow(nextIntent = intent),
                reply = "I do not have a confirmed reservation in this session yet.",
            )

        val workflow = input.session.currentWorkflow ?: workflowDefinition(input.session)?.startSession()
            ?: return ConversationStateResult(
                session = input.session.withoutWorkflow(nextIntent = intent),
                reply = "I do not have a confirmed reservation in this session yet.",
            )

        val result = workflowEngine.advance(
            WorkflowEngineInput(
                ownerIntent = intent,
                incomingIntent = input.intent,
                message = input.message,
                analysis = input.analysis,
                workflow = workflow,
                workflowCommand = input.workflowCommand,
                backgroundEnrichment = input.backgroundEnrichment,
            )
        )

        return when (result.outcome) {
            WorkflowOutcome.IN_PROGRESS,
            WorkflowOutcome.NEEDS_CONFIRMATION -> ConversationStateResult(
                session = input.session.withWorkflow(result.workflow, intent),
                reply = "I found this reservation: ${summary(reservation.filledSlots())}. Should I cancel it?",
            )
            WorkflowOutcome.REJECTED,
            WorkflowOutcome.CANCELLED -> ConversationStateResult(
                session = input.session.withoutWorkflow(nextIntent = intent),
                reply = "No problem. I kept the reservation unchanged.",
                slots = reservation.filledSlots(),
            )
            WorkflowOutcome.CONFIRMED -> ConversationStateResult(
                session = input.session.withoutWorkflow(nextIntent = intent).copy(
                    completedWorkflows = input.session.completedWorkflows -
                        IntentName.RESERVATION_CREATE -
                        IntentName.RESERVATION_MODIFY,
                ),
                reply = "I have cancelled the reservation: ${summary(reservation.filledSlots())}.",
                slots = reservation.filledSlots(),
                completed = true,
            )
        }
    }

    private fun summary(slots: Map<SlotName, String>): String =
        listOfNotNull(
            slots[SlotName.PEOPLE]?.let { "$it people" },
            slots[SlotName.DATE]?.let { "on $it" },
            slots[SlotName.TIME]?.let { "at $it" },
        ).joinToString(" ")
            .let { base -> slots[SlotName.NAME]?.let { if (base.isBlank()) "under $it" else "$base, under $it" } ?: base }
            .ifBlank { "no details captured" }
}
