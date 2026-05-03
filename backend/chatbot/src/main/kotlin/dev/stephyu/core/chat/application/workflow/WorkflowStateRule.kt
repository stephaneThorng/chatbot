package dev.stephyu.core.chat.application.workflow

import dev.stephyu.core.chat.application.state.ProcessingMode
import dev.stephyu.core.chat.domain.intent.IntentName
import dev.stephyu.core.chat.domain.nlp.NlpAnalysis
import dev.stephyu.core.chat.domain.workflow.RequirementParsingContext
import dev.stephyu.core.chat.domain.workflow.WorkflowCommand
import dev.stephyu.core.chat.domain.workflow.WorkflowSession

/**
 * Input consumed by the generic workflow engine for a single turn.
 */
data class WorkflowEngineInput(
    val ownerIntent: IntentName,
    val incomingIntent: IntentName,
    val message: String,
    val analysis: NlpAnalysis,
    val workflow: WorkflowSession,
    val workflowCommand: WorkflowCommand? = null,
    val processingMode: ProcessingMode = ProcessingMode.PRIMARY,
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

/**
 * Applies one workflow lifecycle rule and either updates the rule context or stops execution.
 */
interface WorkflowStateRule {
    fun apply(input: WorkflowEngineInput, context: WorkflowRuleContext): WorkflowRuleResult
}

sealed interface WorkflowRuleResult {
    data class Continue(val context: WorkflowRuleContext) : WorkflowRuleResult
    data class Stop(val result: WorkflowEngineResult) : WorkflowRuleResult
}


