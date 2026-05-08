use thiserror::Error;

use crate::core::nlu_engine::application::nlu_artifacts::{LabelMaps, OnnxContract};
use crate::core::nlu_engine::application::nlu_inference_input::{
    NluInferenceInput, NluModelInference,
};

#[derive(Debug, Error)]
pub enum NluRuntimeError {
    #[error("invalid nlu artifact: {0}")]
    InvalidArtifact(String),
    #[error("onnx runtime error: {0}")]
    Onnx(String),
    #[error("tokenizer error: {0}")]
    Tokenizer(String),
}

pub trait NluModelRuntime: Send + Sync {
    fn contract(&self) -> &OnnxContract;

    fn label_maps(&self) -> &LabelMaps;

    fn run(&self, input: NluInferenceInput) -> Result<NluModelInference, NluRuntimeError>;
}
