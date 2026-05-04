package dev.stephyu.core.chat.application.workflow

import dev.stephyu.core.chat.application.state.ProcessingMode
import dev.stephyu.core.chat.domain.workflow.RequirementName
import dev.stephyu.core.chat.domain.workflow.RequirementParsingResult
import dev.stephyu.core.chat.domain.workflow.WorkflowPhase
import dev.stephyu.core.chat.domain.workflow.WorkflowRequirementSession

/**
 * Binds NLP entities and raw text to the currently active workflow requirements.
 */
class FillRequirementsRule : WorkflowStateRule {
    override fun apply(input: WorkflowEngineInput, context: WorkflowRuleContext): WorkflowRuleResult {
        if (context.workflow.phase != WorkflowPhase.COLLECTING) {
            return WorkflowRuleResult.Continue(context)
        }

        var nextWorkflow = context.workflow
        var invalidMessage = context.invalidMessage

        val pendingRequirements = nextWorkflow.activeRequirements().filterNot { it.isSatisfied() }
        val requirementsToEvaluate = if (input.processingMode == ProcessingMode.BACKGROUND_ENRICHMENT) {
            pendingRequirements.take(1)
        } else {
            pendingRequirements
        }

        for (requirement in requirementsToEvaluate) {
            val result = parseRequirement(requirement, input)
            when (result) {
                is RequirementParsingResult.Valid -> nextWorkflow = nextWorkflow.withRequirementValue(requirement.name, result.value)
                is RequirementParsingResult.Invalid -> invalidMessage = result.message
                RequirementParsingResult.NotMatched -> Unit
            }
        }

        return WorkflowRuleResult.Continue(
            context.copy(
                workflow = nextWorkflow,
                invalidMessage = invalidMessage,
            )
        )
    }

    private fun parseRequirement(
        requirement: WorkflowRequirementSession,
        input: WorkflowEngineInput,
    ): RequirementParsingResult {
        val candidates = candidatesFor(requirement, input)
        var lastInvalid: RequirementParsingResult.Invalid? = null
        for (candidate in candidates) {
            when (val result = requirement.valueType.parse(candidate, input.requirementParsingContext)) {
                is RequirementParsingResult.Valid -> return result
                is RequirementParsingResult.Invalid -> lastInvalid = result
                RequirementParsingResult.NotMatched -> Unit
            }
        }
        return lastInvalid ?: RequirementParsingResult.NotMatched
    }

    private fun candidatesFor(requirement: WorkflowRequirementSession, input: WorkflowEngineInput): List<String> {
        val entityCandidates = input.analysis.entities
            .filter { it.confidence >= ENTITY_CONFIDENCE_THRESHOLD }
            .filter { it.type in requirement.valueType.acceptedEntities }
            .flatMap { listOf(it.value, it.rawValue) }
            .filter { it.isNotBlank() }

        val rawCandidate = input.message.takeIf { shouldTryRawMessage(requirement, input.message, input.workflow) }
        return (entityCandidates + listOfNotNull(rawCandidate)).distinct()
    }

    private fun shouldTryRawMessage(
        requirement: WorkflowRequirementSession,
        message: String,
        contextWorkflow: dev.stephyu.core.chat.domain.workflow.WorkflowSession,
    ): Boolean {
        if (requirement.name != RequirementName.NAME) return true
        val firstMissing = contextWorkflow.firstMissingRequirement()?.name ?: RequirementName.NAME
        return requirement.name == firstMissing || NAME_HINT_PATTERN.containsMatchIn(message)
    }

    companion object {
        private const val ENTITY_CONFIDENCE_THRESHOLD = 0.5
        private val NAME_HINT_PATTERN = Regex("""(?i)\b(under|name is|my name is|au nom de|nom de)\b""")
    }
}


