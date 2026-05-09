use crate::core::nlu_engine::application::AnalyzeTextCommand;
use crate::core::nlu_engine::application::port::output::nlu_model_runtime_trait::NluRuntimeError;
use crate::core::nlu_engine::domain::analysis::NluAnalysis;

/// Inbound application port for text analysis requests.
pub trait AnalyzeText: Send + Sync {
    fn predict(&self, command: AnalyzeTextCommand) -> Result<NluAnalysis, NluRuntimeError>;
}
