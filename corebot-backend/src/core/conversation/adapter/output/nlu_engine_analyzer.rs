use std::sync::Arc;

use crate::core::conversation::application::port::output::nlp_analyzer_trait::NlpAnalyzer;
use crate::core::nlu_engine::application::AnalyzeTextCommand;
use crate::core::nlu_engine::application::port::input::analyze_text_trait::AnalyzeTextNlu;
use crate::core::nlu_engine::domain::analysis::{NluAnalysis, NluIntent};

pub struct NluEngineAnalyzer {
    analyzer: Arc<dyn AnalyzeTextNlu>,
}

impl NluEngineAnalyzer {
    pub fn new(analyzer: Arc<dyn AnalyzeTextNlu>) -> Self {
        Self { analyzer }
    }
}

impl NlpAnalyzer for NluEngineAnalyzer {
    fn analyze(&self, text: &str, lang: &str, domain: &str, task: Option<String>) -> NluAnalysis {
        self.analyzer
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
