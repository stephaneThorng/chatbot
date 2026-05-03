package dev.stephyu.core.chat.config

import dev.stephyu.core.chat.application.intent.IntentService
import dev.stephyu.core.chat.application.intent.IntentPolicy
import dev.stephyu.core.chat.domain.IntentName

class ConversationConfig(
    intentServices: List<IntentService>,
) {
    private val servicesByIntent = intentServices.associateBy { it.intent }
    private val policiesByIntent = intentServices.associate { it.intent to it.policy }

    fun findIntentService(intent: IntentName): IntentService? = servicesByIntent[intent]

    fun findIntentPolicy(intent: IntentName): IntentPolicy = policiesByIntent[intent] ?: IntentPolicy()
}
