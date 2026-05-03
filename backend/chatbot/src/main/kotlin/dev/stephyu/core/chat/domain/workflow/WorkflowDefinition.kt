package dev.stephyu.core.chat.domain.workflow

import dev.stephyu.core.chat.domain.IntentName
import dev.stephyu.core.chat.domain.SlotName
import java.time.Instant

data class WorkflowDefinition(
    val ownerIntent: IntentName,
    val requirements: List<WorkflowRequirementDefinition>,
    val canCancel: Boolean = true,
) {
    fun startSession(): WorkflowSession =
        WorkflowSession(
            ownerIntent = ownerIntent,
            phase = WorkflowPhase.COLLECTING,
            requirements = requirements.map { definition -> WorkflowRequirementSession(definition = definition) },
            canCancel = canCancel,
        )
}

data class WorkflowRequirementDefinition(
    val name: RequirementName,
    val valueType: RequirementValueType,
    val prompt: RequirementPrompt,
    val activation: RequirementActivation = RequirementActivation.ALWAYS,
    val initialValue: RequirementValue? = null,
) {
    val slotName: SlotName? = name.toSlotName()
}

data class WorkflowRequirementSession(
    val definition: WorkflowRequirementDefinition,
    val value: RequirementValue? = definition.initialValue,
) {
    val name: RequirementName = definition.name
    val valueType: RequirementValueType = definition.valueType
    val prompt: RequirementPrompt = definition.prompt
    val activation: RequirementActivation = definition.activation
    val slotName: SlotName? = definition.slotName

    fun isSatisfied(): Boolean = value != null

    fun withValue(nextValue: RequirementValue): WorkflowRequirementSession =
        copy(value = nextValue)
}

data class WorkflowSession(
    val ownerIntent: IntentName,
    val phase: WorkflowPhase,
    val requirements: List<WorkflowRequirementSession>,
    val canCancel: Boolean = true,
) {
    fun activeRequirements(): List<WorkflowRequirementSession> {
        val active = mutableListOf<WorkflowRequirementSession>()
        for (requirement in requirements) {
            val previousSatisfied = active.all { it.isSatisfied() }
            if (requirement.activation == RequirementActivation.ALWAYS || previousSatisfied) {
                active += requirement
            }
        }
        return active
    }

    fun missingRequirements(): List<WorkflowRequirementSession> =
        activeRequirements().filterNot { it.isSatisfied() }

    fun firstMissingRequirement(): WorkflowRequirementSession? =
        missingRequirements().firstOrNull()

    fun filledSlots(): Map<SlotName, String> =
        requirements.mapNotNull { requirement ->
            val slotName = requirement.slotName
            val currentValue = requirement.value
            if (slotName != null && currentValue != null) slotName to currentValue.displayValue else null
        }.toMap()

    fun requiredSlotsForNlp(): List<SlotName> =
        missingRequirements().mapNotNull { requirement ->
            requirement.slotName.takeIf { requirement.valueType.acceptedEntities.isNotEmpty() }
        }

    fun valuesByName(): Map<RequirementName, RequirementValue> =
        requirements.mapNotNull { requirement ->
            requirement.value?.let { requirement.name to it }
        }.toMap()

    fun withRequirementValue(name: RequirementName, value: RequirementValue): WorkflowSession =
        copy(
            requirements = requirements.map { requirement ->
                if (requirement.name == name) requirement.withValue(value) else requirement
            }
        )

    fun clearRequirements(vararg names: RequirementName): WorkflowSession =
        copy(
            requirements = requirements.map { requirement ->
                if (requirement.name in names) requirement.copy(value = null) else requirement
            }
        )

    fun withPhase(nextPhase: WorkflowPhase): WorkflowSession =
        copy(phase = nextPhase)
}

data class WorkflowSnapshot(
    val ownerIntent: IntentName,
    val values: Map<RequirementName, String>,
    val completedAt: Instant,
) {
    fun filledSlots(): Map<SlotName, String> =
        values.mapNotNull { (name, value) ->
            name.toSlotName()?.let { it to value }
        }.toMap()
}
