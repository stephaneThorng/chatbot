package dev.stephyu.core.chat.application.state

/**
 * Describes how the current user message should affect an active workflow.
 */
enum class ProcessingMode {
    PRIMARY,
    BACKGROUND_ENRICHMENT,
}



