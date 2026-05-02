package dev.stephyu.core.chat.domain

enum class SlotName(val wireName: String) {
    DATE("date"),
    TIME("time"),
    PEOPLE("people"),
    NAME("name"),
    PHONE("phone"),
    EMAIL("email"),
    MENU_ITEM("menu_item"),
    PRICE_ITEM("price_item"),
    LOCATION("location");

    companion object {
        fun fromEntityType(entityType: EntityType): SlotName? = when (entityType) {
            EntityType.DATE -> DATE
            EntityType.TIME -> TIME
            EntityType.PEOPLE_COUNT -> PEOPLE
            EntityType.PERSON -> NAME
            EntityType.PHONE -> PHONE
            EntityType.EMAIL -> EMAIL
            EntityType.MENU_ITEM -> MENU_ITEM
            EntityType.PRICE_ITEM -> PRICE_ITEM
            EntityType.LOCATION -> LOCATION
            EntityType.UNKNOWN -> null
        }
    }
}
