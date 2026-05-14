use crate::core::conversation::application::nlu_analysis_result::{
    NluAnalysisResult, NluEntityResult, NluIntentCandidate,
};
use crate::core::conversation::application::port::outbound::nlp_engine_gateway_port::NlpEngineGatewayPort;
use crate::core::conversation::domain::domain_type::DomainType;
use crate::core::conversation::domain::model::intent::NluTask;
use crate::core::nlu_engine::application::AnalyzeTextCommand;
use crate::core::nlu_engine::application::port::inbound::analyze_text_usecase::AnalyzeTextUseCase;
use crate::core::nlu_engine::domain::analysis::NluAnalysis;

/// Outbound gateway from `conversation` to the `nlu_engine` application input port.
pub struct NluEngineGateway<A: AnalyzeTextUseCase> {
    nlu_engine_gateway: A,
}

impl<A: AnalyzeTextUseCase> NluEngineGateway<A> {
    /// Creates the gateway with the target NLU input port implementation.
    pub fn new(analyzer: A) -> Self {
        Self {
            nlu_engine_gateway: analyzer,
        }
    }
}

impl<A: AnalyzeTextUseCase> NlpEngineGatewayPort for NluEngineGateway<A> {
    fn analyze(
        &self,
        text: &str,
        lang: &str,
        domain: DomainType,
        task: Option<NluTask>,
    ) -> NluAnalysisResult {
        let nlu_result = self
            .nlu_engine_gateway
            .analyze(AnalyzeTextCommand {
                text: text.to_string(),
                lang: lang.to_string(),
                domain: domain.as_str().to_string(),
                task: task.map(|t| t.as_tag().to_string()),
            })
            .unwrap_or_else(|error| {
                if debug_nlu_logging_enabled() {
                    println!("[nlu-engine][error] {}", error);
                }
                NluAnalysis {
                    processed_text: text.to_string(),
                    intent: crate::core::nlu_engine::domain::analysis::NluIntent {
                        name: "unknown".to_string(),
                        confidence: 0.0,
                    },
                    intents: vec![],
                    entities: vec![],
                    ner_labels: vec![],
                }
            });
        map_to_result(nlu_result)
    }
}

fn map_to_result(analysis: NluAnalysis) -> NluAnalysisResult {
    NluAnalysisResult {
        intent_name: analysis.intent.name,
        intent_confidence: analysis.intent.confidence,
        intent_candidates: analysis
            .intents
            .into_iter()
            .map(|c| NluIntentCandidate {
                name: c.name,
                confidence: c.confidence,
            })
            .collect(),
        entities: analysis
            .entities
            .into_iter()
            .map(|e| NluEntityResult {
                entity_label: e.entity_type.as_label().to_string(),
                value: e.value,
                raw_value: e.raw_value,
                start: e.start,
                end: e.end,
                confidence: e.confidence,
            })
            .collect(),
    }
}

fn debug_nlu_logging_enabled() -> bool {
    std::env::var("COREBOT_DEBUG_NLU")
        .ok()
        .as_deref()
        .map(is_truthy_env_value)
        .unwrap_or(false)
}

fn is_truthy_env_value(value: &str) -> bool {
    let normalized = value.trim().trim_matches('\'').trim_matches('"');
    matches!(
        normalized.to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}
