package dev.stephyu.core.chat.application.port.out

import dev.stephyu.core.chat.domain.nlp.NlpAnalysis
import dev.stephyu.core.chat.domain.nlp.NlpAnalysisContext

/**
 * Outbound port for NLP intent classification and entity extraction.
 */
interface NlpAnalyzer {
    /**
     * Analyzes one user message for the given domain and optional conversation context.
     */
    suspend fun analyze(text: String, domain: String, context: NlpAnalysisContext?): NlpAnalysis
}


