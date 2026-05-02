package dev.stephyu.core.chat.domain

enum class EntityType(val wireName: String) {
    DATE("DATE"),
    TIME("TIME"),
    PEOPLE_COUNT("PEOPLE_COUNT"),
    PERSON("PERSON"),
    PHONE("PHONE"),
    EMAIL("EMAIL"),
    MENU_ITEM("MENU_ITEM"),
    PRICE_ITEM("PRICE_ITEM"),
    LOCATION("LOCATION"),
    UNKNOWN("UNKNOWN");

    companion object {
        fun fromWireName(value: String?): EntityType =
            entries.firstOrNull { it.wireName == value } ?: UNKNOWN
    }
}
