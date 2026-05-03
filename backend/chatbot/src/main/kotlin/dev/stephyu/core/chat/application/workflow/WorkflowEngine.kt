package dev.stephyu.core.chat.application.workflow

import dev.stephyu.core.chat.domain.workflow.RequirementParsingContext
import dev.stephyu.core.chat.domain.workflow.WorkflowPhase
import java.time.Clock

class WorkflowEngine(
    private val rules: List<WorkflowStateRule>,
    private val clock: Clock,
) {
    fun advance(input: WorkflowEngineInput): WorkflowEngineResult {
        val effectiveInput = input.withParsingContext(
            RequirementParsingContext(
                today = clock.instant().atZone(clock.zone).toLocalDate(),
            )
        )
        var context = WorkflowRuleContext(workflow = input.workflow)
        for (rule in rules) {
            when (val result = rule.apply(effectiveInput, context)) {
                is WorkflowRuleResult.Continue -> context = result.context
                is WorkflowRuleResult.Stop -> return result.result
            }
        }

        return when (context.workflow.phase) {
            WorkflowPhase.COLLECTING -> WorkflowEngineResult(
                workflow = context.workflow,
                outcome = WorkflowOutcome.IN_PROGRESS,
                invalidMessage = context.invalidMessage,
            )

            WorkflowPhase.CONFIRMING -> WorkflowEngineResult(
                workflow = context.workflow,
                outcome = WorkflowOutcome.NEEDS_CONFIRMATION,
                invalidMessage = context.invalidMessage,
            )
        }
    }
}
