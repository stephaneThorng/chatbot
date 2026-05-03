package dev.stephyu.core.chat.domain.intent

import dev.stephyu.core.chat.domain.intent.IntentName

sealed interface IntentDecision {
    data class Accept(
        val intent: IntentName,
        val source: String,
    ) : IntentDecision

    data class Clarify(
        val primary: IntentName,
        val secondary: IntentName,
    ) : IntentDecision

    data object Unknown : IntentDecision
}


