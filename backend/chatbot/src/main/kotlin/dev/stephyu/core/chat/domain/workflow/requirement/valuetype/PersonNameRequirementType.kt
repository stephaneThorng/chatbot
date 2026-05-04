package dev.stephyu.core.chat.domain.workflow.requirement.valuetype

import dev.stephyu.core.chat.domain.nlp.SlotName
import dev.stephyu.core.chat.domain.workflow.RequirementParsingContext
import dev.stephyu.core.chat.domain.workflow.RequirementParsingResult
import dev.stephyu.core.chat.domain.workflow.RequirementValueType
import dev.stephyu.core.chat.domain.workflow.TextRequirementValue

data class PersonNameRequirementType(
    private val minLength: Int = 2,
    private val maxLength: Int = 60,
) : RequirementValueType {
    override val acceptedEntities: Set<SlotName> = setOf(SlotName.NAME)

    override fun parse(raw: String, context: RequirementParsingContext): RequirementParsingResult {
        val candidate = extractName(raw).trim()
        if (candidate.isBlank()) return RequirementParsingResult.NotMatched
        if (candidate.length !in minLength..maxLength) {
            return RequirementParsingResult.Invalid("Please provide a name between $minLength and $maxLength characters.")
        }
        if (candidate.split(Regex("""\s+""")).size > 3) {
            return RequirementParsingResult.NotMatched
        }
        if (Regex("""(?i)\b(reservation|book|booking|cancel|modify|change|opening|hours|menu|price|pricing|contact|phone|email|location|address|need|want|would|like|make|new|please)\b""").containsMatchIn(candidate)) {
            return RequirementParsingResult.NotMatched
        }
        if (!NAME_PATTERN.matches(candidate)) {
            return RequirementParsingResult.Invalid("Please provide a valid reservation name.")
        }
        return RequirementParsingResult.Valid(TextRequirementValue(raw = raw, displayValue = candidate))
    }

    private fun extractName(raw: String): String {
        val trimmed = raw.trim().trimEnd('.', ',', ';')
        NAME_HINT_PATTERN.find(trimmed)?.let { match ->
            return match.groups[1]?.value?.trim()?.trimEnd('.', ',', ';') ?: trimmed
        }
        return trimmed
    }

    companion object {
        private val NAME_PATTERN = Regex("""[A-Za-zÀ-ÖØ-öø-ÿ][A-Za-zÀ-ÖØ-öø-ÿ'\- ]+""")
        private val NAME_HINT_PATTERN = Regex("""(?i)\b(?:under|for|name is|my name is|au nom de|nom de)\s+([A-Za-zÀ-ÖØ-öø-ÿ][A-Za-zÀ-ÖØ-öø-ÿ'\- ]{1,60})\b""")
    }
}