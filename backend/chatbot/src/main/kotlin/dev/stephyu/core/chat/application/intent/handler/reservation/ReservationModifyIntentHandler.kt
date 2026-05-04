package dev.stephyu.core.chat.application.intent.handler.reservation

import dev.stephyu.core.chat.application.port.out.ReservationInventoryRepository
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
import dev.stephyu.core.chat.domain.workflow.DateRequirementType
import dev.stephyu.core.chat.domain.workflow.PartySizeRequirementType
import dev.stephyu.core.chat.domain.workflow.PersonNameRequirementType
import dev.stephyu.core.chat.domain.workflow.RequirementActivation
import dev.stephyu.core.chat.domain.workflow.RequirementName
import dev.stephyu.core.chat.domain.workflow.RequirementPrompt
import dev.stephyu.core.chat.domain.workflow.TextRequirementValue
import dev.stephyu.core.chat.domain.workflow.TimeRequirementType
import dev.stephyu.core.chat.domain.workflow.WorkflowDefinition
import dev.stephyu.core.chat.domain.workflow.WorkflowPhase
import dev.stephyu.core.chat.domain.workflow.WorkflowRequirementDefinition
import dev.stephyu.core.chat.domain.workflow.WorkflowSession
import dev.stephyu.core.chat.domain.workflow.WorkflowSnapshot
import java.time.Clock

class ReservationModifyIntentHandler(
    private val workflowEngine: WorkflowEngine,
    private val inventory: ReservationInventoryRepository,
    private val clock: Clock,
) : IntentHandler {
    override val intent: IntentName = IntentName.RESERVATION_MODIFY
    override val policy: IntentPolicy = IntentPolicy(category = IntentCategory.WORKFLOW)

    override fun workflowDefinition(session: ConversationSession): WorkflowDefinition? {
        val reservation = session.completedWorkflows[IntentName.RESERVATION_MODIFY]
            ?: session.completedWorkflows[IntentName.RESERVATION_CREATE]?: return null

        val name = reservation.values[RequirementName.NAME] ?: return null
        return WorkflowDefinition(
            ownerIntent = intent,
            requirements = requirements(name),
            canCancel = true,
        )
    }

    override fun process(input: StateHandlerInput): StateHandlerResult {
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
            WorkflowOutcome.CANCELLED -> StateHandlerResult(
                updatedSession = input.session.withoutWorkflow(nextIntent = null),
                reply = "I have cancelled the current reservation update request: ${summary(workflow.filledSlots())}.",
                slotSnapshot = workflow.filledSlots(),
            )
            WorkflowOutcome.IN_PROGRESS -> StateHandlerResult(
                updatedSession = input.session.withWorkflow(result.workflow, intent),
                reply = result.invalidMessage ?: result.workflow.firstMissingRequirement()?.prompt?.defaultText.orEmpty(),
            )
            WorkflowOutcome.NEEDS_CONFIRMATION -> StateHandlerResult(
                updatedSession = input.session.withWorkflow(result.workflow, intent),
                reply = confirmationPrompt(result.workflow),
            )
            WorkflowOutcome.REJECTED -> {
                val resetWorkflow = result.workflow
                    .clearRequirements(RequirementName.DATE, RequirementName.TIME, RequirementName.PEOPLE, RequirementName.CONFIRMATION)
                    .withPhase(WorkflowPhase.COLLECTING)
                StateHandlerResult(
                    updatedSession = input.session.withWorkflow(resetWorkflow, intent),
                    reply = "No problem. What new date should I use for the reservation?",
                )
            }
            WorkflowOutcome.CONFIRMED -> confirmReservation(input.session, result.workflow)
        }
    }

    private fun confirmReservation(session: ConversationSession, workflow: WorkflowSession): StateHandlerResult {
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
            return StateHandlerResult(
                updatedSession = session.withWorkflow(nextWorkflow, intent),
                reply = availability.message,
            )
        }

        val snapshot = workflow.toSnapshot(intent, clock)
        return StateHandlerResult(
            updatedSession = session.withoutWorkflow(nextIntent = intent).copy(
                completedWorkflows = session.completedWorkflows + (intent to snapshot),
            ),
            reply = "Your reservation is confirmed: ${summary(slots)}.",
            slotSnapshot = slots,
            completed = true,
        )
    }

    private fun confirmationPrompt(workflow: WorkflowSession): String {
        val slots = workflow.filledSlots()
        return "I have a reservation for ${slots[SlotName.PEOPLE]} people on ${slots[SlotName.DATE]} at ${slots[SlotName.TIME]}, under ${slots[SlotName.NAME]}. Should I confirm it?"
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

    private fun requirements(name: String) = listOf(
        WorkflowRequirementDefinition(
            name = RequirementName.NAME,
            valueType = PersonNameRequirementType(),
            prompt = RequirementPrompt("ask_name", "What name should I use for the reservation?"),
            initialValue = TextRequirementValue(raw = name, displayValue = name),
        ),
        WorkflowRequirementDefinition(
            name = RequirementName.DATE,
            valueType = DateRequirementType,
            prompt = RequirementPrompt("ask_date", "What new date should I use for the reservation?"),
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
    )

    private fun WorkflowSession.toSnapshot(ownerIntent: IntentName, clock: Clock): WorkflowSnapshot =
        WorkflowSnapshot(
            ownerIntent = ownerIntent,
            values = valuesByName()
                .filterKeys { it != RequirementName.CONFIRMATION }
                .mapValues { it.value.displayValue },
            completedAt = clock.instant(),
        )
}


