use thiserror::Error;

use crate::core::nlu_engine::application::nlu_artifacts::{LabelMaps, OnnxContract};
use crate::core::nlu_engine::application::nlu_model_inference::NluModelInference;
use crate::core::nlu_engine::domain::analysis::TaggedInput;

/// Error returned by the NLU runtime adapter or by artifact validation around it.
#[derive(Debug, Error)]
pub enum NluRuntimeError {
    #[error("invalid nlu artifact: {0}")]
    InvalidArtifact(String),
    #[error("onnx runtime error: {0}")]
    Onnx(String),
    #[error("tokenizer error: {0}")]
    Tokenizer(String),
}

/// Output port implemented by concrete NLU runtimes such as ONNX Runtime.
pub trait NluModelRuntime: Send + Sync {
    /// Returns the loaded ONNX contract used to validate output shapes and labels.
    fn contract(&self) -> &OnnxContract;

    /// Returns the loaded label maps used by application-layer decoding.
    fn label_maps(&self) -> &LabelMaps;

    /// Runs inference on an already prepared tagged input and returns raw outputs.
    fn run(&self, input: TaggedInput) -> Result<NluModelInference, NluRuntimeError>;
}
