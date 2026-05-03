package dev.stephyu.core.chat.application.service

import dev.stephyu.core.chat.domain.IntentName

sealed interface IntentRoutingDecision {
    data class Accept(
        val intent: IntentName,
        val source: String,
    ) : IntentRoutingDecision

    data class Clarify(
        val primary: IntentName,
        val secondary: IntentName,
    ) : IntentRoutingDecision

    data object Unknown : IntentRoutingDecision
}
