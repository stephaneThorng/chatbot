package dev.stephyu.core.chat.application.state

interface ChatStateHandler {
    fun process(input: ChatStateInput): ChatStateTransition
}
