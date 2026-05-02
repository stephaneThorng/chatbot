package dev.stephyu.core.chat.application.port.out

import dev.stephyu.core.chat.domain.NlpAnalysis
import dev.stephyu.core.chat.domain.NlpAnalysisContext

interface NlpAnalyzer {
    suspend fun analyze(text: String, domain: String, context: NlpAnalysisContext?): NlpAnalysis
}
