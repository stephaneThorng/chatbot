package dev.stephyu.core.chat.domain.intent

enum class IntentName(val wireName: String) {
    RESERVATION_CREATE("reservation_create"),
    RESERVATION_MODIFY("reservation_modify"),
    RESERVATION_CANCEL("reservation_cancel"),
    RESERVATION_STATUS("reservation_status"),
    MENU_REQUEST("menu_request"),
    OPENING_HOURS("opening_hours"),
    LOCATION_REQUEST("location_request"),
    PRICING_REQUEST("pricing_request"),
    CONTACT_REQUEST("contact_request"),
    UNKNOWN("unknown");

    companion object {
        fun fromWireName(value: String?): IntentName =
            entries.firstOrNull { it.wireName == value } ?: UNKNOWN
    }
}


