package dev.stephyu.core.chat.domain

import dev.stephyu.core.chat.domain.workflow.WorkflowSession
import dev.stephyu.core.chat.domain.workflow.WorkflowSnapshot
import java.time.Instant

data class ConversationSession(
    val id: String,
    val state: ConversationState = ConversationState.IDLE,
    val currentIntent: IntentName? = null,
    val previousIntent: IntentName? = null,
    val lastResolvedInformationalIntent: IntentName? = null,
    val pendingDisambiguation: PendingDisambiguation? = null,
    val currentWorkflow: WorkflowSession? = null,
    val completedWorkflows: Map<IntentName, WorkflowSnapshot> = emptyMap(),
    val createdAt: Instant,
    val updatedAt: Instant,
    val expiresAt: Instant,
) {
    fun hasCurrentWorkflow(): Boolean = currentWorkflow != null

    fun filledSlots(): Map<SlotName, String> =
        currentWorkflow?.filledSlots() ?: emptyMap()

    fun missingSlots(): List<SlotName> =
        currentWorkflow?.missingRequirements()?.mapNotNull { it.slotName } ?: emptyList()

    fun withWorkflow(workflow: WorkflowSession, intent: IntentName = workflow.ownerIntent): ConversationSession =
        copy(
            state = ConversationState.WORKFLOW,
            previousIntent = currentIntent,
            currentIntent = intent,
            pendingDisambiguation = null,
            currentWorkflow = workflow,
        )

    fun withoutWorkflow(nextIntent: IntentName? = currentIntent): ConversationSession =
        copy(
            state = ConversationState.IDLE,
            previousIntent = currentIntent,
            currentIntent = nextIntent,
            pendingDisambiguation = null,
            currentWorkflow = null,
        )

    fun withInformationalIntent(intent: IntentName): ConversationSession =
        copy(
            previousIntent = currentIntent,
            currentIntent = intent,
            lastResolvedInformationalIntent = intent,
            pendingDisambiguation = null,
        )

    fun withPendingDisambiguation(vararg candidates: IntentName): ConversationSession =
        copy(
            previousIntent = currentIntent,
            pendingDisambiguation = PendingDisambiguation(candidates = candidates.toList().distinct()),
        )

    fun clearPendingDisambiguation(): ConversationSession =
        copy(pendingDisambiguation = null)

    fun withRefreshedTimestamps(now: Instant, expiresAt: Instant): ConversationSession =
        copy(updatedAt = now, expiresAt = expiresAt)
}

data class PendingDisambiguation(
    val candidates: List<IntentName>,
)
