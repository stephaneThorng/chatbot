package dev.stephyu.core.chat.application.service

import dev.stephyu.core.chat.domain.ConversationAct

class ConversationActPreprocessor {
    fun preprocess(message: String): PreprocessedMessage {
        var businessText = message.trim()
        var conversationAct: ConversationAct? = null
        var hasLeadingGreeting = false

        LEADING_ACT_PATTERNS.firstNotNullOfOrNull { (act, pattern) ->
            pattern.find(businessText)?.let { act to it }
        }?.let { (act, match) ->
            conversationAct = act
            hasLeadingGreeting = act == ConversationAct.GREETING
            businessText = businessText.removeRange(match.range).trim()
        }

        TRAILING_ACT_PATTERNS.firstNotNullOfOrNull { (act, pattern) ->
            pattern.find(businessText)?.let { act to it }
        }?.let { (act, match) ->
            if (conversationAct == null) {
                conversationAct = act
            }
            businessText = businessText.removeRange(match.range).trim()
        }

        return PreprocessedMessage(
            businessText = businessText.trim(' ', ',', '.', '!', '?', ':', ';', '-'),
            conversationAct = conversationAct,
            hasLeadingGreeting = hasLeadingGreeting,
        )
    }

    companion object {
        private val LEADING_ACT_PATTERNS = listOf(
            ConversationAct.GREETING to Regex("""(?i)^\s*(hello|hi|hey)\b[\s,!.?:;-]*"""),
            ConversationAct.THANKS to Regex("""(?i)^\s*(thanks|thank you|thx|appreciate it)\b[\s,!.?:;-]*"""),
            ConversationAct.FAREWELL to Regex("""(?i)^\s*(talk to you later|see you soon|goodbye|good bye|see you|bye)\b[\s,!.?:;-]*"""),
        )
        private val TRAILING_ACT_PATTERNS = listOf(
            ConversationAct.THANKS to Regex("""(?i)[\s,!.?:;-]*(thanks|thank you|thx|appreciate it)\s*[.!?]*$"""),
            ConversationAct.FAREWELL to Regex("""(?i)[\s,!.?:;-]*(talk to you later|see you soon|goodbye|good bye|see you|bye)\s*[.!?]*$"""),
        )
    }
}

data class PreprocessedMessage(
    val businessText: String,
    val conversationAct: ConversationAct?,
    val hasLeadingGreeting: Boolean,
)
