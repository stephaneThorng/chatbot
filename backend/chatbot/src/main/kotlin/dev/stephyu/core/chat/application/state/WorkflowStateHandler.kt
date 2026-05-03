package dev.stephyu.core.chat.application.state

import dev.stephyu.core.chat.config.ConversationConfig
import dev.stephyu.core.chat.domain.ConversationState
import dev.stephyu.core.chat.domain.IntentName
import dev.stephyu.core.chat.domain.workflow.WorkflowPhase

class WorkflowStateHandler(
    private val conversationConfig: ConversationConfig,
) : StateHandler {
    override fun process(input: ConversationStateInput): ConversationStateResult =
        resolveIntentForState(input)
            ?.let(conversationConfig::findIntentService)
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
            ?: ConversationStateResult(
                session = input.session.copy(state = ConversationState.IDLE),
                reply = "I can help with reservations, opening hours, location, menu, prices, and contact details.",
            )

    private fun resolveIntentForState(input: ConversationStateInput): IntentName? =
        when {
            input.intent in setOf(
                IntentName.MENU_REQUEST,
                IntentName.OPENING_HOURS,
                IntentName.LOCATION_REQUEST,
                IntentName.PRICING_REQUEST,
                IntentName.CONTACT_REQUEST,
            ) -> input.intent
            else -> input.session.currentWorkflow?.ownerIntent
        }

    private fun shouldEnrichWorkflow(input: ConversationStateInput, primaryIntent: IntentName): Boolean =
        input.session.hasCurrentWorkflow() &&
            primaryIntent in INFORMATIONAL_INTENTS &&
            input.session.currentWorkflow?.ownerIntent != null

    private fun enrichWorkflow(
        primaryResult: ConversationStateResult,
        originalInput: ConversationStateInput,
    ): ConversationStateResult {
        val workflowIntent = originalInput.session.currentWorkflow?.ownerIntent ?: return primaryResult
        val workflowService = conversationConfig.findIntentService(workflowIntent) ?: return primaryResult
        val workflowInput = originalInput.copy(
            session = primaryResult.session,
            intent = workflowIntent,
            backgroundEnrichment = true,
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

    private fun workflowResumePrompt(workflowResult: ConversationStateResult): String? {
        val workflow = workflowResult.session.currentWorkflow ?: return null
        return when (workflow.phase) {
            WorkflowPhase.COLLECTING ->
                workflow.firstMissingRequirement()?.prompt?.defaultText?.let { " Next: $it" }
            WorkflowPhase.CONFIRMING -> " Next: Please confirm with yes or no."
        }
    }

    companion object {
        private val INFORMATIONAL_INTENTS = setOf(
            IntentName.MENU_REQUEST,
            IntentName.OPENING_HOURS,
            IntentName.LOCATION_REQUEST,
            IntentName.PRICING_REQUEST,
            IntentName.CONTACT_REQUEST,
        )
    }
}
