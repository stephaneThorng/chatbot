use crate::core::nlu_engine::application::AnalyzeTextCommand;
use crate::core::nlu_engine::application::port::outbound::nlu_model_runtime_trait::NluRuntimeError;
use crate::core::nlu_engine::domain::analysis::NluAnalysis;

/// Inbound application port for text analysis requests.
pub trait AnalyzeTextPort: Send + Sync {
    fn analyze(&self, command: AnalyzeTextCommand) -> Result<NluAnalysis, NluRuntimeError>;
}
