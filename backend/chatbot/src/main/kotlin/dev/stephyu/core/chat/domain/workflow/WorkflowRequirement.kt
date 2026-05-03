package dev.stephyu.core.chat.domain.workflow

enum class RequirementActivation {
    ALWAYS,
    AFTER_PREVIOUS_REQUIREMENTS,
}

data class RequirementPrompt(
    val key: String,
    val defaultText: String,
)


