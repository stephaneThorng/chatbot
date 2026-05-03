package dev.stephyu.core.chat.domain.reservation

/**
 * Static availability rules used by the in-memory reservation inventory.
 */
data class AvailabilityPolicy(
    val minPartySize: Int,
    val maxPartySize: Int,
    val closedDays: Set<String>,
    val unavailableExamples: Set<String>,
)

/**
 * Result returned by the reservation inventory check.
 */
data class AvailabilityResult(
    val available: Boolean,
    val message: String,
)



