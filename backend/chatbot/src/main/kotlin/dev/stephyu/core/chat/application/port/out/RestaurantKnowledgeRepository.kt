package dev.stephyu.core.chat.application.port.out

import dev.stephyu.core.chat.domain.MenuItem
import dev.stephyu.core.chat.domain.OpeningHour
import dev.stephyu.core.chat.domain.PriceInfo
import dev.stephyu.core.chat.domain.RestaurantProfile

interface RestaurantKnowledgeRepository {
    fun profile(): RestaurantProfile
    fun openingHours(): List<OpeningHour>
    fun menuItems(): List<MenuItem>
    fun priceInfo(): List<PriceInfo>
}
