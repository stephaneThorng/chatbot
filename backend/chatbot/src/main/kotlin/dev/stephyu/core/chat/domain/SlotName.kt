package dev.stephyu.core.chat.domain

enum class SlotName(
    val wireName: String,
    private val nlpWireNames: Set<String>,
) {
    DATE("date", setOf("DATE")),
    TIME("time", setOf("TIME")),
    PEOPLE("people", setOf("PEOPLE_COUNT")),
    NAME("name", setOf("PERSON")),
    PHONE("phone", setOf("PHONE")),
    EMAIL("email", setOf("EMAIL")),
    MENU_ITEM("menu_item", setOf("MENU_ITEM")),
    PRICE_ITEM("price_item", setOf("PRICE_ITEM")),
    LOCATION("location", setOf("LOCATION")),
    UNKNOWN("unknown", setOf("UNKNOWN"));

    companion object {
        fun fromNlpWireName(value: String?): SlotName =
            entries.firstOrNull { value in it.nlpWireNames } ?: UNKNOWN
    }
}
