package dev.stephyu.core.chat.application.workflow

import dev.stephyu.core.chat.domain.workflow.WorkflowCommand

class CancelWorkflowRule : WorkflowStateRule {
    override fun apply(input: WorkflowEngineInput, context: WorkflowRuleContext): WorkflowRuleResult {
        val isCancellationRequest = input.workflowCommand == WorkflowCommand.CANCEL &&
            context.workflow.canCancel
        if (!isCancellationRequest) {
            return WorkflowRuleResult.Continue(context)
        }
        return WorkflowRuleResult.Stop(
            WorkflowEngineResult(
                workflow = context.workflow,
                outcome = WorkflowOutcome.CANCELLED,
                invalidMessage = context.invalidMessage,
            )
        )
    }
}


