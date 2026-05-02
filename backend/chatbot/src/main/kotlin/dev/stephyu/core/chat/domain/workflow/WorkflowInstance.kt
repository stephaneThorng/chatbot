package dev.stephyu.core.chat.domain.workflow

import dev.stephyu.core.chat.domain.SlotName
import java.time.Instant

data class WorkflowInstance(
    val name: WorkflowName,
    val requirements: List<WorkflowRequirement>,
    val canCancel: Boolean = true,
) {
    fun activeRequirements(): List<WorkflowRequirement> {
        val active = mutableListOf<WorkflowRequirement>()
        for (requirement in requirements) {
            val previousSatisfied = active.all { it.isSatisfied() }
            if (requirement.activation == RequirementActivation.ALWAYS || previousSatisfied) {
                active += requirement
            }
        }
        return active
    }

    fun missingRequirements(): List<WorkflowRequirement> =
        activeRequirements().filterNot { it.isSatisfied() }

    fun firstMissingRequirement(): WorkflowRequirement? =
        missingRequirements().firstOrNull()

    fun valuesByName(): Map<RequirementName, RequirementValue> =
        requirements.mapNotNull { requirement ->
            requirement.value?.let { requirement.name to it }
        }.toMap()

    fun wireSlots(): Map<SlotName, String> =
        requirements.mapNotNull { requirement ->
            val slotName = requirement.slotName
            val value = requirement.value
            if (slotName != null && value != null) slotName to value.displayValue else null
        }.toMap()

    fun requiredSlotsForNlp(): List<SlotName> =
        missingRequirements().mapNotNull { requirement ->
            requirement.slotName.takeIf { requirement.valueType.acceptedEntities.isNotEmpty() }
        }

    fun withRequirementValue(name: RequirementName, value: RequirementValue): WorkflowInstance =
        copy(
            requirements = requirements.map { requirement ->
                if (requirement.name == name) requirement.withValue(value) else requirement
            }
        )
}

data class WorkflowSnapshot(
    val name: WorkflowName,
    val values: Map<RequirementName, String>,
    val completedAt: Instant,
) {
    fun wireSlots(): Map<SlotName, String> =
        values.mapNotNull { (name, value) ->
            name.toSlotName()?.let { it to value }
        }.toMap()
}
