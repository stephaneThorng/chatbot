package dev.stephyu.core.chat.domain.knowledge

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


