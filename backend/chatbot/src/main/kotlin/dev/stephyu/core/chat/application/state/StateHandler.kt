package dev.stephyu.core.chat.application.state

interface StateHandler {
    fun process(input: ConversationStateInput): ConversationStateResult
}
