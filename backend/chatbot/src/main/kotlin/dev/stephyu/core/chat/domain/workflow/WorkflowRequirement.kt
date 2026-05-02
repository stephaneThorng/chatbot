package dev.stephyu.core.chat.domain.workflow

import dev.stephyu.core.chat.domain.SlotName

enum class RequirementActivation {
    ALWAYS,
    AFTER_PREVIOUS_REQUIREMENTS
}

data class RequirementPrompt(
    val key: String,
    val defaultText: String,
)

data class WorkflowRequirement(
    val name: RequirementName,
    val valueType: RequirementValueType,
    val prompt: RequirementPrompt,
    val value: RequirementValue? = null,
    val activation: RequirementActivation = RequirementActivation.ALWAYS,
) {
    val slotName: SlotName? = name.toSlotName()

    fun isSatisfied(): Boolean = value != null

    fun withValue(nextValue: RequirementValue): WorkflowRequirement =
        copy(value = nextValue)
}
