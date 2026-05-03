package dev.stephyu.core.chat.application.state

import dev.stephyu.core.chat.application.intent.catalog.IntentCatalog
import dev.stephyu.core.chat.application.intent.policy.IntentCategory
import dev.stephyu.core.chat.domain.conversation.ConversationState
import dev.stephyu.core.chat.domain.intent.IntentName
import dev.stephyu.core.chat.domain.workflow.WorkflowPhase

class WorkflowStateHandler(
    private val intentCatalog: IntentCatalog,
) : StateHandler {
    override fun process(input: ConversationTurnContext): ConversationTurnResult =
        resolveIntentForState(input)
            ?.let(intentCatalog::findIntentHandler)
            ?.let { primaryService ->
                val primaryResult = primaryService.process(input).copy(
                    handledIntent = primaryService.intent,
                )
                if (!shouldEnrichWorkflow(input, primaryService.intent)) {
                    primaryResult
                } else {
                    enrichWorkflow(primaryResult, input)
                }
            }
            ?: ConversationTurnResult(
                session = input.session.copy(state = ConversationState.IDLE),
                reply = "I can help with reservations, opening hours, location, menu, prices, and contact details.",
            )

    private fun resolveIntentForState(input: ConversationTurnContext): IntentName? =
        when {
            intentCatalog.findIntentPolicy(input.intent).allowDuringWorkflow -> input.intent
            else -> input.session.currentWorkflow?.ownerIntent
        }

    private fun shouldEnrichWorkflow(input: ConversationTurnContext, primaryIntent: IntentName): Boolean =
        input.session.hasCurrentWorkflow() &&
            intentCatalog.findIntentPolicy(primaryIntent).category == IntentCategory.INFORMATIONAL &&
            input.session.currentWorkflow?.ownerIntent != null

    private fun enrichWorkflow(
        primaryResult: ConversationTurnResult,
        originalInput: ConversationTurnContext,
    ): ConversationTurnResult {
        val workflowIntent = originalInput.session.currentWorkflow?.ownerIntent ?: return primaryResult
        val workflowService = intentCatalog.findIntentHandler(workflowIntent) ?: return primaryResult
        val workflowInput = originalInput.copy(
            session = primaryResult.session,
            intent = workflowIntent,
            processingMode = ProcessingMode.BACKGROUND_ENRICHMENT,
        )
        val workflowResult = workflowService.process(workflowInput)
        val resumePrompt = workflowResumePrompt(workflowResult)

        return primaryResult.copy(
            session = workflowResult.session,
            reply = primaryResult.reply + resumePrompt.orEmpty(),
            slots = workflowResult.slots,
            missingSlots = workflowResult.missingSlots,
        )
    }

    private fun workflowResumePrompt(workflowResult: ConversationTurnResult): String? {
        val workflow = workflowResult.session.currentWorkflow ?: return null
        return when (workflow.phase) {
            WorkflowPhase.COLLECTING ->
                workflow.firstMissingRequirement()?.prompt?.defaultText?.let { " Next: $it" }
            WorkflowPhase.CONFIRMING -> " Next: Please confirm with yes or no."
        }
    }

}


