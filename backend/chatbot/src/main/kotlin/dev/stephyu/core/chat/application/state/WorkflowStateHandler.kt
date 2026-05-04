package dev.stephyu.core.chat.application.state

import dev.stephyu.core.chat.application.intent.catalog.IntentCatalog
import dev.stephyu.core.chat.application.intent.policy.IntentCategory
import dev.stephyu.core.chat.domain.conversation.ConversationState
import dev.stephyu.core.chat.domain.intent.IntentName
import dev.stephyu.core.chat.domain.workflow.WorkflowPhase

class WorkflowStateHandler(
    private val intentCatalog: IntentCatalog,
) : StateHandler {
    override fun process(input: StateHandlerInput): StateHandlerResult {
        val targetIntent = resolveTargetIntent(input)
            ?: return fallbackToIdle(input)

        val intentHandler = intentCatalog.findIntentHandler(targetIntent)
            ?: return fallbackToIdle(input)

        val result = intentHandler
            .process(input)
            .copy(handledIntentOverride = intentHandler.intent)

        if (!shouldEnrichWorkflow(input, intentHandler.intent)) {
            return result
        }

        return enrichWorkflow(result, input)
    }

    private fun resolveTargetIntent(input: StateHandlerInput): IntentName? {
        val inputIntentPolicy = intentCatalog.findIntentPolicy(input.intent)
        if (inputIntentPolicy.allowDuringWorkflow) {
            return input.intent
        }
        return input.session.currentWorkflow?.ownerIntent
    }

    private fun fallbackToIdle(input: StateHandlerInput): StateHandlerResult =
        StateHandlerResult(
            updatedSession = input.session.copy(state = ConversationState.IDLE),
            reply = "I can help with reservations, opening hours, location, menu, prices, and contact details.",
        )

    private fun shouldEnrichWorkflow(input: StateHandlerInput, primaryIntent: IntentName): Boolean =
        input.session.hasCurrentWorkflow() &&
            intentCatalog.findIntentPolicy(primaryIntent).category == IntentCategory.INFORMATIONAL &&
            input.session.currentWorkflow?.ownerIntent != null

    private fun enrichWorkflow(
        primaryResult: StateHandlerResult,
        originalInput: StateHandlerInput,
    ): StateHandlerResult {
        val workflowIntent = originalInput.session.currentWorkflow?.ownerIntent
            ?: return primaryResult

        val workflowService = intentCatalog.findIntentHandler(workflowIntent)
            ?: return primaryResult

        val workflowInput = originalInput.copy(
            session = primaryResult.updatedSession,
            intent = workflowIntent,
            processingMode = ProcessingMode.BACKGROUND_ENRICHMENT,
        )
        val workflowResult = workflowService.process(workflowInput)
        val resumePrompt = workflowResumePrompt(workflowResult)

        return primaryResult.copy(
            updatedSession = workflowResult.updatedSession,
            reply = primaryResult.reply + resumePrompt.orEmpty(),
            slotSnapshot = workflowResult.slotSnapshot,
            missingSlotSnapshot = workflowResult.missingSlotSnapshot,
        )
    }

    private fun workflowResumePrompt(workflowResult: StateHandlerResult): String? {
        val workflow = workflowResult.updatedSession.currentWorkflow
            ?: return null

        return when (workflow.phase) {
            WorkflowPhase.COLLECTING ->
                workflow.firstMissingRequirement()?.prompt?.defaultText?.let { " Next: $it" }
            WorkflowPhase.CONFIRMING -> " Next: Please confirm with yes or no."
        }
    }

}


