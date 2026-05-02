package dev.stephyu.core.chat.adapter.out.memory

import dev.stephyu.core.chat.application.port.out.ReservationInventoryRepository
import dev.stephyu.core.chat.domain.AvailabilityPolicy
import dev.stephyu.core.chat.domain.AvailabilityResult

class InMemoryReservationInventoryRepository : ReservationInventoryRepository {
    private val policy = AvailabilityPolicy(
        minPartySize = 1,
        maxPartySize = 12,
        closedDays = setOf("sunday"),
        unavailableExamples = setOf("friday 20:00", "vendredi 20h"),
    )

    override fun checkAvailability(date: String, time: String, people: String): AvailabilityResult {
        val partySize = people.filter { it.isDigit() }.toIntOrNull()
            ?: return AvailabilityResult(false, "I could not read the party size. For how many people?")

        if (partySize !in policy.minPartySize..policy.maxPartySize) {
            return AvailabilityResult(
                available = false,
                message = "We can accept parties from ${policy.minPartySize} to ${policy.maxPartySize} people. For how many people should I book?",
            )
        }

        val normalizedDate = date.lowercase()
        if (policy.closedDays.any { it in normalizedDate }) {
            return AvailabilityResult(false, "We are closed on that day. What other date would work?")
        }

        val normalizedSlot = "$normalizedDate ${time.lowercase()}"
        if (policy.unavailableExamples.any { it in normalizedSlot }) {
            return AvailabilityResult(false, "That slot is not available in the demo inventory. What other time would work?")
        }

        return AvailabilityResult(true, "Available")
    }
}
