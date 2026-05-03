package dev.stephyu.core.chat.application.port.out

import dev.stephyu.core.chat.domain.knowledge.MenuItem
import dev.stephyu.core.chat.domain.knowledge.OpeningHour
import dev.stephyu.core.chat.domain.knowledge.PriceInfo
import dev.stephyu.core.chat.domain.knowledge.RestaurantProfile

/**
 * Outbound repository port for static restaurant knowledge used by informational intents.
 */
interface RestaurantKnowledgeRepository {
    fun profile(): RestaurantProfile
    fun openingHours(): List<OpeningHour>
    fun menuItems(): List<MenuItem>
    fun priceInfo(): List<PriceInfo>
}


