package dev.stephyu.core.chat.application.intent.handler.reservation

import dev.stephyu.core.chat.application.state.StateHandlerInput
import dev.stephyu.core.chat.application.state.StateHandlerResult
import dev.stephyu.core.chat.application.intent.handler.IntentHandler
import dev.stephyu.core.chat.application.intent.policy.IntentCategory
import dev.stephyu.core.chat.application.intent.policy.IntentPolicy
import dev.stephyu.core.chat.application.workflow.WorkflowEngine
import dev.stephyu.core.chat.application.workflow.WorkflowEngineInput
import dev.stephyu.core.chat.application.workflow.WorkflowOutcome
import dev.stephyu.core.chat.domain.session.ConversationSession
import dev.stephyu.core.chat.domain.intent.IntentName
import dev.stephyu.core.chat.domain.nlp.SlotName
import dev.stephyu.core.chat.domain.workflow.ConfirmationRequirementType
import dev.stephyu.core.chat.domain.workflow.RequirementActivation
import dev.stephyu.core.chat.domain.workflow.RequirementName
import dev.stephyu.core.chat.domain.workflow.RequirementPrompt
import dev.stephyu.core.chat.domain.workflow.WorkflowDefinition
import dev.stephyu.core.chat.domain.workflow.WorkflowRequirementDefinition

class ReservationCancelIntentHandler(
    private val workflowEngine: WorkflowEngine,
) : IntentHandler {
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

    override fun process(input: StateHandlerInput): StateHandlerResult {
        val reservation = input.session.completedWorkflows[IntentName.RESERVATION_MODIFY]
            ?: input.session.completedWorkflows[IntentName.RESERVATION_CREATE]
            ?: return StateHandlerResult(
                updatedSession = input.session.withoutWorkflow(nextIntent = intent),
                reply = "I do not have a confirmed reservation in this session yet.",
            )

        val workflow = input.session.currentWorkflow ?: workflowDefinition(input.session)?.startSession()
            ?: return StateHandlerResult(
                updatedSession = input.session.withoutWorkflow(nextIntent = intent),
                reply = "I do not have a confirmed reservation in this session yet.",
            )

        val result = workflowEngine.advance(
            WorkflowEngineInput(
                ownerIntent = intent,
                incomingIntent = input.intent,
                message = input.processedText,
                analysis = input.analysis,
                workflow = workflow,
                workflowCommand = input.workflowCommand,
                processingMode = input.processingMode,
            )
        )

        return when (result.outcome) {
            WorkflowOutcome.IN_PROGRESS,
            WorkflowOutcome.NEEDS_CONFIRMATION -> StateHandlerResult(
                updatedSession = input.session.withWorkflow(result.workflow, intent),
                reply = "I found this reservation: ${summary(reservation.filledSlots())}. Should I cancel it?",
            )
            WorkflowOutcome.REJECTED,
            WorkflowOutcome.CANCELLED -> StateHandlerResult(
                updatedSession = input.session.withoutWorkflow(nextIntent = intent),
                reply = "No problem. I kept the reservation unchanged.",
                slotSnapshot = reservation.filledSlots(),
            )
            WorkflowOutcome.CONFIRMED -> StateHandlerResult(
                updatedSession = input.session.withoutWorkflow(nextIntent = intent).copy(
                    completedWorkflows = input.session.completedWorkflows -
                        IntentName.RESERVATION_CREATE -
                        IntentName.RESERVATION_MODIFY,
                ),
                reply = "I have cancelled the reservation: ${summary(reservation.filledSlots())}.",
                slotSnapshot = reservation.filledSlots(),
                completed = true,
            )
        }
    }

    private fun summary(slots: Map<SlotName, String>): String {
        val parts = buildList {
            slots[SlotName.PEOPLE]?.let { add("$it people") }
            slots[SlotName.DATE]?.let { add("on $it") }
            slots[SlotName.TIME]?.let { add("at $it") }
            slots[SlotName.NAME]?.let { add("under $it") }
        }.joinToString(" ")

        return parts.ifBlank { "no details captured" }
    }
}


