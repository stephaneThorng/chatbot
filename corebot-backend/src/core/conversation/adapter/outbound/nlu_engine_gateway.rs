use std::sync::Arc;

use crate::core::conversation::application::port::outbound::nlp_analyzer_trait::NlpEngineGatewayPort;
use crate::core::nlu_engine::application::AnalyzeTextCommand;
use crate::core::nlu_engine::application::port::inbound::analyze_text_trait::AnalyzeTextPort;
use crate::core::nlu_engine::domain::analysis::{NluAnalysis, NluIntent};

/// Outbound gateway from `conversation` to the `nlu_engine` application input port.
pub struct NluEngineGateway {
    nlu_engine_analyzer: Arc<dyn AnalyzeTextPort>,
}

impl NluEngineGateway {
    /// Creates the gateway with the target NLU input port implementation.
    pub fn new(analyzer: Arc<dyn AnalyzeTextPort>) -> Self {
        Self { nlu_engine_analyzer: analyzer }
    }
}

impl NlpEngineGatewayPort for NluEngineGateway {
    fn analyze(&self, text: &str, lang: &str, domain: &str, task: Option<String>) -> NluAnalysis {
        self.nlu_engine_analyzer
            .analyze(AnalyzeTextCommand {
                text: text.to_string(),
                lang: lang.to_string(),
                domain: domain.to_string(),
                task,
            })
            .unwrap_or_else(|_| NluAnalysis {
                tagged_text: text.to_string(),
                intent: NluIntent {
                    name: "unknown".to_string(),
                    confidence: 0.0,
                },
                intents: vec![],
                entities: vec![],
                ner_labels: vec![],
            })
    }
}
