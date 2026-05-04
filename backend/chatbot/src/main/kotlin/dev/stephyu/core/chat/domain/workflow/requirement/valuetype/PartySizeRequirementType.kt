package dev.stephyu.core.chat.domain.workflow.requirement.valuetype

import dev.stephyu.core.chat.domain.nlp.SlotName
import dev.stephyu.core.chat.domain.workflow.IntegerRequirementValue
import dev.stephyu.core.chat.domain.workflow.RequirementParsingContext
import dev.stephyu.core.chat.domain.workflow.RequirementParsingResult
import dev.stephyu.core.chat.domain.workflow.RequirementValueType

data object PartySizeRequirementType : RequirementValueType {
    override val acceptedEntities: Set<SlotName> = setOf(SlotName.PEOPLE)

    override fun parse(raw: String, context: RequirementParsingContext): RequirementParsingResult {
        val candidate = PEOPLE_PATTERN.find(raw)?.groups?.drop(1)?.firstNotNullOfOrNull { it?.value }
            ?: raw.trim().takeIf { PEOPLE_ONLY_PATTERN.matches(it) }
            ?: return RequirementParsingResult.NotMatched
        val value = candidate.toIntOrNull() ?: return RequirementParsingResult.NotMatched
        if (value !in context.minPartySize..context.maxPartySize) {
            return RequirementParsingResult.Invalid(
                "We can accept parties from ${context.minPartySize} to ${context.maxPartySize} people. For how many people should I book?"
            )
        }
        return RequirementParsingResult.Valid(IntegerRequirementValue(raw = raw, value = value, displayValue = value.toString()))
    }

    private val PEOPLE_PATTERN = Regex("""(?i)\b(?:for\s*)?(\d{1,3})\s*(?:people|persons|guests|personnes|couverts)\b|\bfor\s*(\d{1,3})\b""")
    private val PEOPLE_ONLY_PATTERN = Regex("""\s*\d{1,3}\s*""")
}