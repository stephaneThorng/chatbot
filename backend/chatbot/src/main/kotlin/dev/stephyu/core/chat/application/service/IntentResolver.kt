package dev.stephyu.core.chat.application.service

import dev.stephyu.core.chat.domain.ConversationSession
import dev.stephyu.core.chat.domain.IntentName
import dev.stephyu.core.chat.domain.NlpAnalysis

class IntentResolver(
    private val confidenceThreshold: Double = 0.6,
) {
    fun resolve(analysis: NlpAnalysis, session: ConversationSession): IntentName {
        if (session.hasActiveReservationWorkflow() &&
            (analysis.intent.name == IntentName.UNKNOWN || analysis.intent.confidence < confidenceThreshold)
        ) {
            return session.currentWorkflow?.name?.toIntentName() ?: IntentName.RESERVATION_CREATE
        }
        return if (analysis.intent.confidence >= confidenceThreshold) {
            analysis.intent.name
        } else {
            IntentName.UNKNOWN
        }
    }
}
