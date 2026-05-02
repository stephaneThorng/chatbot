package dev.stephyu.core.chat.application.service

import dev.stephyu.core.chat.application.port.out.ReservationInventoryRepository
import dev.stephyu.core.chat.domain.ConversationSession
import dev.stephyu.core.chat.domain.ConversationState
import dev.stephyu.core.chat.domain.IntentName
import dev.stephyu.core.chat.domain.NlpEntity
import dev.stephyu.core.chat.domain.SlotName
import dev.stephyu.core.chat.domain.reservation.ReservationTransition
import dev.stephyu.core.chat.domain.workflow.BooleanRequirementValue
import dev.stephyu.core.chat.domain.workflow.ConfirmationRequirementType
import dev.stephyu.core.chat.domain.workflow.DateRequirementType
import dev.stephyu.core.chat.domain.workflow.PartySizeRequirementType
import dev.stephyu.core.chat.domain.workflow.PersonNameRequirementType
import dev.stephyu.core.chat.domain.workflow.RequirementActivation
import dev.stephyu.core.chat.domain.workflow.RequirementName
import dev.stephyu.core.chat.domain.workflow.RequirementParsingContext
import dev.stephyu.core.chat.domain.workflow.RequirementParsingResult
import dev.stephyu.core.chat.domain.workflow.RequirementPrompt
import dev.stephyu.core.chat.domain.workflow.TextRequirementValue
import dev.stephyu.core.chat.domain.workflow.TimeRequirementType
import dev.stephyu.core.chat.domain.workflow.WorkflowInstance
import dev.stephyu.core.chat.domain.workflow.WorkflowInput
import dev.stephyu.core.chat.domain.workflow.WorkflowName
import dev.stephyu.core.chat.domain.workflow.WorkflowRequirement
import dev.stephyu.core.chat.domain.workflow.WorkflowSnapshot
import java.time.Clock

class ReservationWorkflowService(
    private val inventory: ReservationInventoryRepository,
    private val replies: ReplyComposer,
    private val clock: Clock,
) {
    fun handle(input: WorkflowInput): ReservationTransition {
        val currentWorkflow = input.session.currentWorkflow
        if (currentWorkflow != null &&
            currentWorkflow.name != WorkflowName.RESERVATION_CANCELLATION &&
            input.intent == IntentName.RESERVATION_CANCEL &&
            currentWorkflow.canCancel
        ) {
            return abortCurrentWorkflow(input.session)
        }

        val workflow = currentWorkflow ?: startWorkflow(input.session, input.intent)
            ?: return noReservation(input.session, input.intent)

        return progressWorkflow(input, workflow)
    }

    fun status(session: ConversationSession): ReservationTransition {
        val reservation = session.currentReservationSnapshot()
            ?: return ReservationTransition(
                session.copy(state = ConversationState.IDLE, currentIntent = IntentName.RESERVATION_STATUS),
                "I do not have a confirmed reservation in this session yet.",
            )
        return ReservationTransition(
            session.copy(state = ConversationState.IDLE, currentIntent = IntentName.RESERVATION_STATUS),
            "Your reservation is confirmed: ${replies.reservationSummary(reservation.wireSlots())}.",
            slots = reservation.wireSlots(),
        )
    }

    private fun startWorkflow(session: ConversationSession, intent: IntentName): WorkflowInstance? =
        when (intent) {
            IntentName.RESERVATION_CREATE -> reservationCreationWorkflow()
            IntentName.RESERVATION_MODIFY -> reservationModificationWorkflow(session)
            IntentName.RESERVATION_CANCEL -> reservationCancellationWorkflow(session)
            else -> null
        }

    private fun progressWorkflow(input: WorkflowInput, workflow: WorkflowInstance): ReservationTransition {
        val parsingContext = RequirementParsingContext(today = clock.instant().atZone(clock.zone).toLocalDate())
        var nextWorkflow = workflow
        var invalidMessage: String? = null

        for (requirement in nextWorkflow.activeRequirements().filterNot { it.isSatisfied() }) {
            val result = parseRequirement(requirement, input, parsingContext)
            when (result) {
                is RequirementParsingResult.Valid -> nextWorkflow = nextWorkflow.withRequirementValue(requirement.name, result.value)
                is RequirementParsingResult.Invalid -> invalidMessage = result.message
                RequirementParsingResult.NotMatched -> Unit
            }
        }

        val missing = nextWorkflow.firstMissingRequirement()
        if (missing != null) {
            val reply = invalidMessage ?: promptForMissingRequirement(input.session, nextWorkflow, missing.name)
            return ReservationTransition(input.session.withWorkflow(nextWorkflow), reply)
        }

        return completeWorkflow(input.session, nextWorkflow)
    }

    private fun parseRequirement(
        requirement: WorkflowRequirement,
        input: WorkflowInput,
        context: RequirementParsingContext,
    ): RequirementParsingResult {
        val candidates = candidatesFor(requirement, input)
        var lastInvalid: RequirementParsingResult.Invalid? = null
        for (candidate in candidates) {
            when (val result = requirement.valueType.parse(candidate, context)) {
                is RequirementParsingResult.Valid -> return result
                is RequirementParsingResult.Invalid -> lastInvalid = result
                RequirementParsingResult.NotMatched -> Unit
            }
        }
        return lastInvalid ?: RequirementParsingResult.NotMatched
    }

    private fun candidatesFor(requirement: WorkflowRequirement, input: WorkflowInput): List<String> {
        val entityCandidates = input.analysis.entities
            .filter { it.confidence >= ENTITY_CONFIDENCE_THRESHOLD }
            .filter { it.type in requirement.valueType.acceptedEntities }
            .map(NlpEntity::value)

        val rawCandidate = input.message.takeIf { shouldTryRawMessage(requirement, input) }
        return (entityCandidates + listOfNotNull(rawCandidate)).distinct()
    }

    private fun shouldTryRawMessage(requirement: WorkflowRequirement, input: WorkflowInput): Boolean {
        if (requirement.name != RequirementName.NAME) return true
        if (input.session.currentWorkflow == null && !input.message.containsNameHint()) return false
        val firstMissing = input.session.currentWorkflow?.firstMissingRequirement()?.name ?: RequirementName.NAME
        return requirement.name == firstMissing || input.message.containsNameHint()
    }

    private fun completeWorkflow(session: ConversationSession, workflow: WorkflowInstance): ReservationTransition {
        val confirmation = workflow.valuesByName()[RequirementName.CONFIRMATION] as? BooleanRequirementValue
        if (confirmation?.value != true) {
            return handleRejectedConfirmation(session, workflow)
        }

        return when (workflow.name) {
            WorkflowName.RESERVATION_CREATION,
            WorkflowName.RESERVATION_MODIFICATION -> confirmReservationWorkflow(session, workflow)
            WorkflowName.RESERVATION_CANCELLATION -> confirmCancellationWorkflow(session, workflow)
        }
    }

    private fun confirmReservationWorkflow(session: ConversationSession, workflow: WorkflowInstance): ReservationTransition {
        val slots = workflow.wireSlots()
        val availability = inventory.checkAvailability(
            date = slots.getValue(SlotName.DATE),
            time = slots.getValue(SlotName.TIME),
            people = slots.getValue(SlotName.PEOPLE),
        )
        if (!availability.available) {
            val nextWorkflow = workflow.clear(RequirementName.TIME, RequirementName.CONFIRMATION)
            return ReservationTransition(session.withWorkflow(nextWorkflow), availability.message)
        }

        val snapshot = workflow.toSnapshot(clock.instant())
        val completedWorkflows = session.completedWorkflows +
            (WorkflowName.RESERVATION_CREATION to snapshot) +
            (workflow.name to snapshot)
        return ReservationTransition(
            session.copy(
                state = ConversationState.IDLE,
                previousIntent = session.currentIntent,
                currentIntent = workflow.name.toIntentName(),
                currentWorkflow = null,
                completedWorkflows = completedWorkflows,
            ),
            "Your reservation is confirmed: ${replies.reservationSummary(slots)}.",
            slots = slots,
            completed = true,
        )
    }

    private fun confirmCancellationWorkflow(session: ConversationSession, workflow: WorkflowInstance): ReservationTransition {
        val reservation = session.currentReservationSnapshot()
            ?: return noReservation(session, IntentName.RESERVATION_CANCEL)
        return ReservationTransition(
            session.copy(
                state = ConversationState.IDLE,
                previousIntent = session.currentIntent,
                currentIntent = IntentName.RESERVATION_CANCEL,
                currentWorkflow = null,
                completedWorkflows = session.completedWorkflows -
                    WorkflowName.RESERVATION_CREATION -
                    WorkflowName.RESERVATION_MODIFICATION,
            ),
            "I have cancelled the reservation: ${replies.reservationSummary(reservation.wireSlots())}.",
            slots = reservation.wireSlots(),
            completed = true,
        )
    }

    private fun handleRejectedConfirmation(session: ConversationSession, workflow: WorkflowInstance): ReservationTransition {
        if (workflow.name == WorkflowName.RESERVATION_CANCELLATION) {
            return ReservationTransition(
                session.copy(state = ConversationState.IDLE, currentWorkflow = null, currentIntent = IntentName.RESERVATION_CANCEL),
                "No problem. I kept the reservation unchanged.",
                slots = session.currentReservationSnapshot()?.wireSlots() ?: emptyMap(),
            )
        }
        val nextWorkflow = workflow.clear(RequirementName.DATE, RequirementName.TIME, RequirementName.PEOPLE, RequirementName.CONFIRMATION)
        return ReservationTransition(
            session.withWorkflow(nextWorkflow),
            "No problem. What date would you like to reserve?",
        )
    }

    private fun abortCurrentWorkflow(session: ConversationSession): ReservationTransition {
        val slots = session.currentWorkflow?.wireSlots() ?: emptyMap()
        return ReservationTransition(
            session.copy(
                state = ConversationState.IDLE,
                previousIntent = session.currentIntent,
                currentIntent = null,
                currentWorkflow = null,
            ),
            "I have cancelled the current reservation request: ${replies.reservationSummary(slots)}.",
            slots = slots,
        )
    }

    private fun noReservation(session: ConversationSession, intent: IntentName): ReservationTransition =
        ReservationTransition(
            session.copy(state = ConversationState.IDLE, currentIntent = intent, currentWorkflow = null),
            "I do not have a confirmed reservation in this session yet.",
        )

    private fun ConversationSession.withWorkflow(workflow: WorkflowInstance): ConversationSession =
        copy(
            state = workflow.name.toConversationState(),
            previousIntent = currentIntent,
            currentIntent = workflow.name.toIntentName(),
            currentWorkflow = workflow,
        )

    private fun reservationCreationWorkflow(): WorkflowInstance =
        WorkflowInstance(
            name = WorkflowName.RESERVATION_CREATION,
            requirements = reservationDataRequirements() + confirmationRequirement(),
        )

    private fun reservationModificationWorkflow(session: ConversationSession): WorkflowInstance? {
        val reservation = session.currentReservationSnapshot() ?: return null
        val name = reservation.values[RequirementName.NAME] ?: return null
        return WorkflowInstance(
            name = WorkflowName.RESERVATION_MODIFICATION,
            requirements = reservationDataRequirements(prefilledName = name) + confirmationRequirement(),
        )
    }

    private fun reservationCancellationWorkflow(session: ConversationSession): WorkflowInstance? {
        session.currentReservationSnapshot() ?: return null
        return WorkflowInstance(
            name = WorkflowName.RESERVATION_CANCELLATION,
            requirements = listOf(confirmationRequirement(activation = RequirementActivation.ALWAYS)),
        )
    }

    private fun reservationDataRequirements(prefilledName: String? = null): List<WorkflowRequirement> =
        listOf(
            WorkflowRequirement(
                name = RequirementName.NAME,
                valueType = PersonNameRequirementType(),
                prompt = RequirementPrompt("ask_name", "What name should I use for the reservation?"),
                value = prefilledName?.let { TextRequirementValue(raw = it, displayValue = it) },
            ),
            WorkflowRequirement(
                name = RequirementName.DATE,
                valueType = DateRequirementType,
                prompt = RequirementPrompt("ask_date", "What date would you like to reserve?"),
            ),
            WorkflowRequirement(
                name = RequirementName.TIME,
                valueType = TimeRequirementType,
                prompt = RequirementPrompt("ask_time", "What time would you like?"),
            ),
            WorkflowRequirement(
                name = RequirementName.PEOPLE,
                valueType = PartySizeRequirementType,
                prompt = RequirementPrompt("ask_people", "For how many people?"),
            ),
        )

    private fun confirmationRequirement(
        activation: RequirementActivation = RequirementActivation.AFTER_PREVIOUS_REQUIREMENTS,
    ): WorkflowRequirement =
        WorkflowRequirement(
            name = RequirementName.CONFIRMATION,
            valueType = ConfirmationRequirementType,
            prompt = RequirementPrompt("ask_confirmation", "Please confirm with yes or no."),
            activation = activation,
        )

    private fun promptForMissingRequirement(
        session: ConversationSession,
        workflow: WorkflowInstance,
        requirementName: RequirementName,
    ): String =
        when {
            requirementName == RequirementName.CONFIRMATION && workflow.name == WorkflowName.RESERVATION_CANCELLATION -> {
                val summary = session.currentReservationSnapshot()?.wireSlots()?.let(replies::reservationSummary)
                    ?: "the current reservation"
                "I found this reservation: $summary. Should I cancel it?"
            }
            requirementName == RequirementName.CONFIRMATION -> replies.confirmationPrompt(workflow.wireSlots())
            else -> workflow.firstMissingRequirement()?.prompt?.defaultText ?: replies.unknownReply()
        }

    private fun WorkflowInstance.clear(vararg names: RequirementName): WorkflowInstance =
        copy(requirements = requirements.map { requirement ->
            if (requirement.name in names) requirement.copy(value = null) else requirement
        })

    private fun WorkflowInstance.toSnapshot(completedAt: java.time.Instant): WorkflowSnapshot =
        WorkflowSnapshot(
            name = name,
            values = valuesByName()
                .filterKeys { it != RequirementName.CONFIRMATION }
                .mapValues { it.value.displayValue },
            completedAt = completedAt,
        )

    private fun String.containsNameHint(): Boolean =
        Regex("""(?i)\b(under|name is|my name is|au nom de|nom de)\b""").containsMatchIn(this)

    companion object {
        private const val ENTITY_CONFIDENCE_THRESHOLD = 0.5
    }
}
