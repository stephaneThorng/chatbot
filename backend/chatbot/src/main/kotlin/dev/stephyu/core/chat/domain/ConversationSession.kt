package dev.stephyu.core.chat.domain

import dev.stephyu.core.chat.domain.workflow.WorkflowInstance
import dev.stephyu.core.chat.domain.workflow.WorkflowName
import dev.stephyu.core.chat.domain.workflow.WorkflowSnapshot
import java.time.Instant

data class ConversationSession(
    val id: String,
    val state: ConversationState = ConversationState.IDLE,
    val currentIntent: IntentName? = null,
    val previousIntent: IntentName? = null,
    val currentWorkflow: WorkflowInstance? = null,
    val completedWorkflows: Map<WorkflowName, WorkflowSnapshot> = emptyMap(),
    val createdAt: Instant,
    val updatedAt: Instant,
    val expiresAt: Instant,
) {
    fun hasCurrentWorkflow(): Boolean = currentWorkflow != null

    fun currentSlots(): Map<SlotName, String> =
        currentWorkflow?.wireSlots() ?: emptyMap()

    fun missingCurrentSlots(): List<SlotName> =
        currentWorkflow?.missingRequirements()?.mapNotNull { it.slotName } ?: emptyList()

    fun currentReservationSnapshot(): WorkflowSnapshot? =
        completedWorkflows[WorkflowName.RESERVATION_CREATION]
            ?: completedWorkflows[WorkflowName.RESERVATION_MODIFICATION]

    fun withRefreshedTimestamps(now: Instant, expiresAt: Instant): ConversationSession =
        copy(updatedAt = now, expiresAt = expiresAt)
}
