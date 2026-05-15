use crate::core::nlu_engine::application::AnalyzeTextCommand;
use crate::core::nlu_engine::application::port::outbound::nlu_model_runtime_port::NluRuntimeError;
use crate::core::nlu_engine::domain::analysis::NluAnalysis;

/// Inbound application port for text analysis requests.
pub trait AnalyzeTextUseCase {
    fn analyze(&self, command: AnalyzeTextCommand) -> Result<NluAnalysis, NluRuntimeError>;
}
