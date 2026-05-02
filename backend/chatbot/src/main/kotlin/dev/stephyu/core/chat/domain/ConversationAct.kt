package dev.stephyu.core.chat.domain

enum class ConversationAct(val wireName: String) {
    GREETING("greeting"),
    THANKS("thanks"),
    FAREWELL("farewell")
}
