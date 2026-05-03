package dev.stephyu.core.chat.application.intent.policy

import dev.stephyu.core.chat.domain.nlp.SlotName

data class IntentPolicy(
    val category: IntentCategory = IntentCategory.OTHER,
    val clarifiable: Boolean = false,
    val supportsTopicContinuation: Boolean = false,
    val allowDuringWorkflow: Boolean = false,
    val entitySupport: Set<SlotName> = emptySet(),
    val disambiguationLabels: List<String> = emptyList(),
)

enum class IntentCategory {
    WORKFLOW,
    INFORMATIONAL,
    STATUS,
    OTHER,
}


