package dev.stephyu.core.chat.domain.workflow

import dev.stephyu.core.chat.domain.ConversationState
import dev.stephyu.core.chat.domain.IntentName

enum class WorkflowName {
    RESERVATION_CREATION,
    RESERVATION_MODIFICATION,
    RESERVATION_CANCELLATION;

    fun toConversationState(): ConversationState = when (this) {
        RESERVATION_CREATION -> ConversationState.RESERVATION_CREATION
        RESERVATION_MODIFICATION -> ConversationState.RESERVATION_MODIFICATION
        RESERVATION_CANCELLATION -> ConversationState.RESERVATION_CANCELLATION
    }

    fun toIntentName(): IntentName = when (this) {
        RESERVATION_CREATION -> IntentName.RESERVATION_CREATE
        RESERVATION_MODIFICATION -> IntentName.RESERVATION_MODIFY
        RESERVATION_CANCELLATION -> IntentName.RESERVATION_CANCEL
    }

    companion object {
        fun fromIntent(intent: IntentName): WorkflowName? = when (intent) {
            IntentName.RESERVATION_CREATE -> RESERVATION_CREATION
            IntentName.RESERVATION_MODIFY -> RESERVATION_MODIFICATION
            IntentName.RESERVATION_CANCEL -> RESERVATION_CANCELLATION
            else -> null
        }
    }
}
