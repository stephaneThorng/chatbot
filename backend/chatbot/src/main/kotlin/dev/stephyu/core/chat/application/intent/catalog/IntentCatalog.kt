package dev.stephyu.core.chat.application.intent.catalog

import dev.stephyu.core.chat.application.intent.handler.IntentHandler
import dev.stephyu.core.chat.application.intent.policy.IntentPolicy
import dev.stephyu.core.chat.domain.intent.IntentName

/**
 * Registry for intent handlers and their orchestration policies.
 */
class IntentCatalog(
    intentServices: List<IntentHandler>,
) {
    private val servicesByIntent = intentServices.associateBy { it.intent }
    private val policiesByIntent = intentServices.associate { it.intent to it.policy }

    /**
     * Returns the handler responsible for the given business intent.
     */
    fun findIntentHandler(intent: IntentName): IntentHandler? = servicesByIntent[intent]

    /**
     * Returns the policy metadata used by routing and workflow coordination for the given intent.
     */
    fun findIntentPolicy(intent: IntentName): IntentPolicy = policiesByIntent[intent] ?: IntentPolicy()
}


