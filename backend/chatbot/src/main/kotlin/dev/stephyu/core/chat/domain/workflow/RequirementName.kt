package dev.stephyu.core.chat.domain.workflow

import dev.stephyu.core.chat.domain.SlotName

enum class RequirementName {
    NAME,
    DATE,
    TIME,
    PEOPLE,
    CONFIRMATION;

    fun toSlotName(): SlotName? = when (this) {
        NAME -> SlotName.NAME
        DATE -> SlotName.DATE
        TIME -> SlotName.TIME
        PEOPLE -> SlotName.PEOPLE
        CONFIRMATION -> null
    }
}
