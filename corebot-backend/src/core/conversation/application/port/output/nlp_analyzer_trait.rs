use crate::core::nlu_engine::domain::analysis::NluAnalysis;

pub trait NlpAnalyzer: Send + Sync {
    fn analyze(&self, text: &str, lang: &str, domain: &str, task: Option<String>) -> NluAnalysis;
}
