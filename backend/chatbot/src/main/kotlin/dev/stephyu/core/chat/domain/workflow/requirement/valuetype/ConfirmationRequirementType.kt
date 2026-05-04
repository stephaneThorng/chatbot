package dev.stephyu.core.chat.domain.workflow.requirement.valuetype

import dev.stephyu.core.chat.domain.nlp.SlotName
import dev.stephyu.core.chat.domain.workflow.BooleanRequirementValue
import dev.stephyu.core.chat.domain.workflow.RequirementParsingContext
import dev.stephyu.core.chat.domain.workflow.RequirementParsingResult
import dev.stephyu.core.chat.domain.workflow.RequirementValueType
import java.util.Locale

data object ConfirmationRequirementType : RequirementValueType {
    override val acceptedEntities: Set<SlotName> = emptySet()

    override fun parse(raw: String, context: RequirementParsingContext): RequirementParsingResult {
        val normalized = raw.trim().lowercase(Locale.ROOT)
        if (isYes(normalized)) {
            return RequirementParsingResult.Valid(BooleanRequirementValue(raw = raw, value = true, displayValue = "yes"))
        }
        if (isNo(normalized)) {
            return RequirementParsingResult.Valid(BooleanRequirementValue(raw = raw, value = false, displayValue = "no"))
        }
        return RequirementParsingResult.NotMatched
    }

    private fun isYes(normalized: String): Boolean =
        normalized in setOf("yes", "y", "ok", "okay", "confirm", "confirmed", "sure", "oui") ||
            normalized.startsWith("yes ") ||
            "confirm it" in normalized ||
            "you can confirm" in normalized

    private fun isNo(normalized: String): Boolean =
        normalized in setOf("no", "n", "nope", "change", "non") ||
            normalized.startsWith("no ")
}