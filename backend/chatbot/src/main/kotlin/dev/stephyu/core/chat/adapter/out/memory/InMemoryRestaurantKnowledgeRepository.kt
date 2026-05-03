package dev.stephyu.core.chat.adapter.out.memory

import dev.stephyu.core.chat.application.port.out.RestaurantKnowledgeRepository
import dev.stephyu.core.chat.domain.knowledge.MenuItem
import dev.stephyu.core.chat.domain.knowledge.OpeningHour
import dev.stephyu.core.chat.domain.knowledge.PriceInfo
import dev.stephyu.core.chat.domain.knowledge.RestaurantProfile

class InMemoryRestaurantKnowledgeRepository : RestaurantKnowledgeRepository {
    override fun profile(): RestaurantProfile =
        RestaurantProfile(
            name = "Maison Demo",
            address = "12 Rue de la Paix, 75002 Paris",
            phone = "+33 1 42 00 00 00",
            email = "contact@maison-demo.example",
            locationHints = listOf("Near Opera metro station", "Ten minutes from Palais Garnier"),
            parkingHints = listOf("Public parking is available at Parking Meyerbeer Opera."),
        )

    override fun openingHours(): List<OpeningHour> =
        listOf(
            OpeningHour("Monday", "12:00", "22:00"),
            OpeningHour("Tuesday", "12:00", "22:00"),
            OpeningHour("Wednesday", "12:00", "22:00"),
            OpeningHour("Thursday", "12:00", "23:00"),
            OpeningHour("Friday", "12:00", "23:30"),
            OpeningHour("Saturday", "11:30", "23:30"),
            OpeningHour("Sunday", null, null),
        )

    override fun menuItems(): List<MenuItem> =
        listOf(
            MenuItem("Burrata Tomatoes", "starter", "Creamy burrata, marinated tomatoes, basil oil", "12 EUR", setOf("vegetarian")),
            MenuItem("Lentil Veloute", "starter", "Green lentil soup with herbs and croutons", "9 EUR", setOf("vegan")),
            MenuItem("Seared Sea Bass", "main", "Sea bass with seasonal vegetables and lemon butter", "24 EUR", setOf("seafood", "fish")),
            MenuItem("Truffle Risotto", "main", "Arborio rice, mushrooms, parmesan, black truffle", "22 EUR", setOf("vegetarian")),
            MenuItem("Chocolate Fondant", "dessert", "Warm chocolate cake with vanilla ice cream", "10 EUR"),
            MenuItem("Seasonal Fruit Pavlova", "dessert", "Meringue, whipped cream, and seasonal fruit", "9 EUR"),
            MenuItem("House Lemonade", "drink", "Fresh lemon, mint, sparkling water", "6 EUR"),
            MenuItem("French Red Wine Glass", "drink", "Rotating regional red wine by the glass", "8 EUR"),
        )

    override fun priceInfo(): List<PriceInfo> =
        listOf(
            PriceInfo("Starters", "9-12 EUR"),
            PriceInfo("Mains", "22-24 EUR"),
            PriceInfo("Desserts", "9-10 EUR"),
            PriceInfo("Drinks", "6-8 EUR"),
            PriceInfo("Average dinner", "35-45 EUR per person without wine pairing"),
        )
}


