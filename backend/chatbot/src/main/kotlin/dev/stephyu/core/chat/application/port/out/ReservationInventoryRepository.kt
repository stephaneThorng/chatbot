package dev.stephyu.core.chat.application.port.out

import dev.stephyu.core.chat.domain.reservation.AvailabilityResult

/**
 * Outbound port for reservation availability checks.
 */
interface ReservationInventoryRepository {
    /**
     * Validates whether a reservation slot is acceptable for the demo inventory.
     */
    fun checkAvailability(date: String, time: String, people: String): AvailabilityResult
}


