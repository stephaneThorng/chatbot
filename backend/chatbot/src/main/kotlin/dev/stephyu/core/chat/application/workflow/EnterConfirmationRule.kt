package dev.stephyu.core.chat.application.workflow

import dev.stephyu.core.chat.domain.workflow.RequirementName
import dev.stephyu.core.chat.domain.workflow.WorkflowPhase

class EnterConfirmationRule : WorkflowStateRule {
    override fun apply(input: WorkflowEngineInput, context: WorkflowRuleContext): WorkflowRuleResult {
        if (context.workflow.phase != WorkflowPhase.COLLECTING) {
            return WorkflowRuleResult.Continue(context)
        }
        val remainingBusinessRequirements = context.workflow.missingRequirements()
            .filterNot { it.name == RequirementName.CONFIRMATION }
        if (remainingBusinessRequirements.isNotEmpty()) {
            return WorkflowRuleResult.Continue(context)
        }
        return WorkflowRuleResult.Continue(
            context.copy(workflow = context.workflow.withPhase(WorkflowPhase.CONFIRMING))
        )
    }
}


