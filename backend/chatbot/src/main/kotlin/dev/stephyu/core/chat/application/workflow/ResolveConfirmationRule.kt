package dev.stephyu.core.chat.application.workflow

import dev.stephyu.core.chat.domain.workflow.BooleanRequirementValue
import dev.stephyu.core.chat.domain.workflow.RequirementName
import dev.stephyu.core.chat.domain.workflow.RequirementParsingResult
import dev.stephyu.core.chat.domain.workflow.WorkflowPhase

class ResolveConfirmationRule : WorkflowStateRule {
    override fun apply(input: WorkflowEngineInput, context: WorkflowRuleContext): WorkflowRuleResult {
        if (context.workflow.phase != WorkflowPhase.CONFIRMING) {
            return WorkflowRuleResult.Continue(context)
        }

        val confirmationRequirement = context.workflow.requirements
            .firstOrNull { it.name == RequirementName.CONFIRMATION }
            ?: return WorkflowRuleResult.Stop(
                WorkflowEngineResult(
                    workflow = context.workflow,
                    outcome = WorkflowOutcome.NEEDS_CONFIRMATION,
                    invalidMessage = context.invalidMessage,
                )
            )

        return when (val result = confirmationRequirement.valueType.parse(input.message, input.requirementParsingContext)) {
            is RequirementParsingResult.Valid -> {
                val value = result.value as? BooleanRequirementValue
                val nextWorkflow = context.workflow.withRequirementValue(RequirementName.CONFIRMATION, result.value)
                if (value?.value == true) {
                    WorkflowRuleResult.Stop(
                        WorkflowEngineResult(
                            workflow = nextWorkflow,
                            outcome = WorkflowOutcome.CONFIRMED,
                            invalidMessage = context.invalidMessage,
                        )
                    )
                } else {
                    WorkflowRuleResult.Stop(
                        WorkflowEngineResult(
                            workflow = nextWorkflow,
                            outcome = WorkflowOutcome.REJECTED,
                            invalidMessage = context.invalidMessage,
                        )
                    )
                }
            }
            is RequirementParsingResult.Invalid -> WorkflowRuleResult.Stop(
                WorkflowEngineResult(
                    workflow = context.workflow,
                    outcome = WorkflowOutcome.NEEDS_CONFIRMATION,
                    invalidMessage = result.message,
                )
            )
            RequirementParsingResult.NotMatched -> WorkflowRuleResult.Stop(
                WorkflowEngineResult(
                    workflow = context.workflow,
                    outcome = WorkflowOutcome.NEEDS_CONFIRMATION,
                    invalidMessage = context.invalidMessage,
                )
            )
        }
    }
}


