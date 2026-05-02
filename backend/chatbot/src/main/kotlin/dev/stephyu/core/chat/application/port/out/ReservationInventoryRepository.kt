package dev.stephyu.core.chat.application.port.out

import dev.stephyu.core.chat.domain.AvailabilityResult

interface ReservationInventoryRepository {
    fun checkAvailability(date: String, time: String, people: String): AvailabilityResult
}
