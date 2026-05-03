package dev.stephyu.core.chat.application.intent

import dev.stephyu.core.chat.application.port.out.ReservationInventoryRepository
import dev.stephyu.core.chat.application.state.ConversationStateInput
import dev.stephyu.core.chat.application.state.ConversationStateResult
import dev.stephyu.core.chat.application.workflow.WorkflowEngine
import dev.stephyu.core.chat.application.workflow.WorkflowEngineInput
import dev.stephyu.core.chat.application.workflow.WorkflowOutcome
import dev.stephyu.core.chat.domain.ConversationSession
import dev.stephyu.core.chat.domain.IntentName
import dev.stephyu.core.chat.domain.SlotName
import dev.stephyu.core.chat.domain.workflow.ConfirmationRequirementType
import dev.stephyu.core.chat.domain.workflow.DateRequirementType
import dev.stephyu.core.chat.domain.workflow.PartySizeRequirementType
import dev.stephyu.core.chat.domain.workflow.PersonNameRequirementType
import dev.stephyu.core.chat.domain.workflow.RequirementActivation
import dev.stephyu.core.chat.domain.workflow.RequirementName
import dev.stephyu.core.chat.domain.workflow.RequirementPrompt
import dev.stephyu.core.chat.domain.workflow.TimeRequirementType
import dev.stephyu.core.chat.domain.workflow.WorkflowDefinition
import dev.stephyu.core.chat.domain.workflow.WorkflowPhase
import dev.stephyu.core.chat.domain.workflow.WorkflowRequirementDefinition
import dev.stephyu.core.chat.domain.workflow.WorkflowSession
import dev.stephyu.core.chat.domain.workflow.WorkflowSnapshot
import java.time.Clock

class ReservationCreateIntentService(
    private val workflowEngine: WorkflowEngine,
    private val inventory: ReservationInventoryRepository,
    private val clock: Clock,
) : IntentService {
    override val intent: IntentName = IntentName.RESERVATION_CREATE
    override val policy: IntentPolicy = IntentPolicy(category = IntentCategory.WORKFLOW)

    override fun workflowDefinition(session: ConversationSession): WorkflowDefinition =
        WorkflowDefinition(
            ownerIntent = intent,
            requirements = listOf(
                WorkflowRequirementDefinition(
                    name = RequirementName.NAME,
                    valueType = PersonNameRequirementType(),
                    prompt = RequirementPrompt("ask_name", "What name should I use for the reservation?"),
                ),
                WorkflowRequirementDefinition(
                    name = RequirementName.DATE,
                    valueType = DateRequirementType,
                    prompt = RequirementPrompt("ask_date", "What date would you like to reserve?"),
                ),
                WorkflowRequirementDefinition(
                    name = RequirementName.TIME,
                    valueType = TimeRequirementType,
                    prompt = RequirementPrompt("ask_time", "What time would you like?"),
                ),
                WorkflowRequirementDefinition(
                    name = RequirementName.PEOPLE,
                    valueType = PartySizeRequirementType,
                    prompt = RequirementPrompt("ask_people", "For how many people?"),
                ),
                WorkflowRequirementDefinition(
                    name = RequirementName.CONFIRMATION,
                    valueType = ConfirmationRequirementType,
                    prompt = RequirementPrompt("ask_confirmation", "Please confirm with yes or no."),
                    activation = RequirementActivation.AFTER_PREVIOUS_REQUIREMENTS,
                ),
            ),
            canCancel = true,
        )

    override fun process(input: ConversationStateInput): ConversationStateResult {
        val workflow = input.session.currentWorkflow ?: workflowDefinition(input.session).startSession()
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
            WorkflowOutcome.CANCELLED -> ConversationStateResult(
                session = input.session.withoutWorkflow(nextIntent = null),
                reply = "I have cancelled the current reservation request: ${summary(workflow.filledSlots())}.",
                slots = workflow.filledSlots(),
            )
            WorkflowOutcome.IN_PROGRESS -> ConversationStateResult(
                session = input.session.withWorkflow(result.workflow, intent),
                reply = result.invalidMessage ?: result.workflow.firstMissingRequirement()?.prompt?.defaultText.orEmpty(),
            )
            WorkflowOutcome.NEEDS_CONFIRMATION -> ConversationStateResult(
                session = input.session.withWorkflow(result.workflow, intent),
                reply = confirmationPrompt(result.workflow),
            )
            WorkflowOutcome.REJECTED -> {
                val resetWorkflow = result.workflow
                    .clearRequirements(RequirementName.DATE, RequirementName.TIME, RequirementName.PEOPLE, RequirementName.CONFIRMATION)
                    .withPhase(WorkflowPhase.COLLECTING)
                ConversationStateResult(
                    session = input.session.withWorkflow(resetWorkflow, intent),
                    reply = "No problem. What date would you like to reserve?",
                )
            }
            WorkflowOutcome.CONFIRMED -> confirmReservation(input.session, result.workflow)
        }
    }

    private fun confirmReservation(session: ConversationSession, workflow: WorkflowSession): ConversationStateResult {
        val slots = workflow.filledSlots()
        val availability = inventory.checkAvailability(
            date = slots.getValue(SlotName.DATE),
            time = slots.getValue(SlotName.TIME),
            people = slots.getValue(SlotName.PEOPLE),
        )
        if (!availability.available) {
            val nextWorkflow = workflow
                .clearRequirements(RequirementName.TIME, RequirementName.CONFIRMATION)
                .withPhase(WorkflowPhase.COLLECTING)
            return ConversationStateResult(
                session = session.withWorkflow(nextWorkflow, intent),
                reply = availability.message,
            )
        }

        val snapshot = workflow.toSnapshot(intent, clock)
        return ConversationStateResult(
            session = session.withoutWorkflow(nextIntent = intent).copy(
                completedWorkflows = session.completedWorkflows + (intent to snapshot),
            ),
            reply = "Your reservation is confirmed: ${summary(slots)}.",
            slots = slots,
            completed = true,
        )
    }

    private fun confirmationPrompt(workflow: WorkflowSession): String {
        val slots = workflow.filledSlots()
        return "I have a reservation for ${slots[SlotName.PEOPLE]} people on ${slots[SlotName.DATE]} at ${slots[SlotName.TIME]}, under ${slots[SlotName.NAME]}. Should I confirm it?"
    }

    private fun summary(slots: Map<SlotName, String>): String =
        listOfNotNull(
            slots[SlotName.PEOPLE]?.let { "$it people" },
            slots[SlotName.DATE]?.let { "on $it" },
            slots[SlotName.TIME]?.let { "at $it" },
        ).joinToString(" ")
            .let { base -> slots[SlotName.NAME]?.let { if (base.isBlank()) "under $it" else "$base, under $it" } ?: base }
            .ifBlank { "no details captured" }

    private fun WorkflowSession.toSnapshot(ownerIntent: IntentName, clock: Clock): WorkflowSnapshot =
        WorkflowSnapshot(
            ownerIntent = ownerIntent,
            values = valuesByName()
                .filterKeys { it != RequirementName.CONFIRMATION }
                .mapValues { it.value.displayValue },
            completedAt = clock.instant(),
        )
}
