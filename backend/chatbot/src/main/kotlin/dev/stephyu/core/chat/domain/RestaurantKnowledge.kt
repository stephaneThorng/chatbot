package dev.stephyu.core.chat.domain

data class RestaurantProfile(
    val name: String,
    val address: String,
    val phone: String,
    val email: String,
    val locationHints: List<String>,
    val parkingHints: List<String>,
)

data class OpeningHour(
    val day: String,
    val opensAt: String?,
    val closesAt: String?,
)

data class MenuItem(
    val name: String,
    val category: String,
    val description: String,
    val price: String,
    val tags: Set<String> = emptySet(),
)

data class PriceInfo(
    val label: String,
    val value: String,
)

data class AvailabilityPolicy(
    val minPartySize: Int,
    val maxPartySize: Int,
    val closedDays: Set<String>,
    val unavailableExamples: Set<String>,
)

data class ReservationContext(
    val name: String,
    val date: String,
    val time: String,
    val people: String,
)

data class AvailabilityResult(
    val available: Boolean,
    val message: String,
)
