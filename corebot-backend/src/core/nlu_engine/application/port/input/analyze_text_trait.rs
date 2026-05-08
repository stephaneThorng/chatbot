use crate::core::nlu_engine::application::AnalyzeTextCommand;
use crate::core::nlu_engine::application::port::output::nlu_model_runtime_trait::NluRuntimeError;
use crate::core::nlu_engine::domain::analysis::NluAnalysis;

pub trait AnalyzeTextNlu: Send + Sync {
    fn analyze(&self, command: AnalyzeTextCommand) -> Result<NluAnalysis, NluRuntimeError>;
}
