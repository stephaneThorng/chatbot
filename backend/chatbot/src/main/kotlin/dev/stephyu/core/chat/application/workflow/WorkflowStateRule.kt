package dev.stephyu.core.chat.application.workflow

import dev.stephyu.core.chat.domain.IntentName
import dev.stephyu.core.chat.domain.NlpAnalysis
import dev.stephyu.core.chat.domain.workflow.WorkflowCommand
import dev.stephyu.core.chat.domain.workflow.RequirementParsingContext
import dev.stephyu.core.chat.domain.workflow.WorkflowSession

data class WorkflowEngineInput(
    val ownerIntent: IntentName,
    val incomingIntent: IntentName,
    val message: String,
    val analysis: NlpAnalysis,
    val workflow: WorkflowSession,
    val workflowCommand: WorkflowCommand? = null,
    val backgroundEnrichment: Boolean = false,
    val parsingContext: RequirementParsingContext? = null,
) {
    fun withParsingContext(context: RequirementParsingContext): WorkflowEngineInput =
        copy(parsingContext = context)
}

val WorkflowEngineInput.requirementParsingContext: RequirementParsingContext
    get() = parsingContext ?: error("Requirement parsing context must be set by WorkflowEngine.")

data class WorkflowRuleContext(
    val workflow: WorkflowSession,
    val invalidMessage: String? = null,
)

data class WorkflowEngineResult(
    val workflow: WorkflowSession,
    val outcome: WorkflowOutcome,
    val invalidMessage: String? = null,
)

enum class WorkflowOutcome {
    IN_PROGRESS,
    NEEDS_CONFIRMATION,
    CONFIRMED,
    REJECTED,
    CANCELLED,
}

interface WorkflowStateRule {
    fun apply(input: WorkflowEngineInput, context: WorkflowRuleContext): WorkflowRuleResult
}

sealed interface WorkflowRuleResult {
    data class Continue(val context: WorkflowRuleContext) : WorkflowRuleResult
    data class Stop(val result: WorkflowEngineResult) : WorkflowRuleResult
}
