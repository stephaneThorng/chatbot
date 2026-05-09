use std::fs;
use std::path::Path;
use std::sync::Mutex;

use ndarray::Array2;
use ort::session::Session;
use ort::value::TensorRef;
use tokenizers::Tokenizer;
use tokenizers::utils::truncation::TruncationDirection;

use crate::core::nlu_engine::application::nlu_artifacts::{LabelMaps, OnnxContract};
use crate::core::nlu_engine::application::nlu_model_inference::{
    NluModelInference, OnnxModelOutput,
};
use crate::core::nlu_engine::application::port::output::nlu_model_runtime_trait::{
    NluModelRuntime, NluRuntimeError,
};
use crate::core::nlu_engine::domain::analysis::TaggedInput;

/// Concrete output adapter backed by ONNX Runtime and the exported tokenizer.
pub struct OnnxNluRuntime {
    session: Mutex<Session>,
    tokenizer: Tokenizer,
    contract: OnnxContract,
    label_maps: LabelMaps,
}

struct EncodedInput {
    input_ids: Array2<i64>,
    attention_mask: Array2<i64>,
    tokens: Vec<String>,
    offsets: Vec<(usize, usize)>,
}

impl OnnxNluRuntime {
    /// Loads the tokenizer, ONNX contract, label maps, and model session from an artifact directory.
    pub fn from_artifact_dir(artifact_dir: impl AsRef<Path>) -> Result<Self, NluRuntimeError> {
        let artifact_dir = artifact_dir.as_ref();
        let model_path = artifact_dir.join("model.onnx");
        let tokenizer_path = artifact_dir.join("tokenizer.json");
        let contract_path = artifact_dir.join("onnx_contract.json");
        let label_maps_path = artifact_dir.join("label_maps.json");

        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|error| NluRuntimeError::Tokenizer(error.to_string()))?;

        let contract: OnnxContract =
            serde_json::from_str(&fs::read_to_string(&contract_path).map_err(|error| {
                NluRuntimeError::InvalidArtifact(format!("{contract_path:?}: {error}"))
            })?)
            .map_err(|error| {
                NluRuntimeError::InvalidArtifact(format!("{contract_path:?}: {error}"))
            })?;

        let label_maps: LabelMaps =
            serde_json::from_str(&fs::read_to_string(&label_maps_path).map_err(|error| {
                NluRuntimeError::InvalidArtifact(format!("{label_maps_path:?}: {error}"))
            })?)
            .map_err(|error| {
                NluRuntimeError::InvalidArtifact(format!("{label_maps_path:?}: {error}"))
            })?;

        let session = Session::builder()
            .map_err(|error| NluRuntimeError::Onnx(error.to_string()))?
            .commit_from_file(&model_path)
            .map_err(|error| NluRuntimeError::Onnx(error.to_string()))?;

        Ok(Self {
            session: Mutex::new(session),
            tokenizer,
            contract,
            label_maps,
        })
    }

    /// Loads the runtime from the `COREBOT_NLU_ONNX_DIR` environment variable.
    pub fn from_env() -> Result<Self, NluRuntimeError> {
        match std::env::var("COREBOT_NLU_ONNX_DIR") {
            Ok(path) => Self::from_artifact_dir(path),
            Err(std::env::VarError::NotPresent) => Err(NluRuntimeError::InvalidArtifact(
                "COREBOT_NLU_ONNX_DIR is not set".to_string(),
            )),
            Err(error) => Err(NluRuntimeError::InvalidArtifact(error.to_string())),
        }
    }

    fn encode_input(&self, input: TaggedInput) -> Result<EncodedInput, NluRuntimeError> {
        let mut encoding = self
            .tokenizer
            .encode(input.text, true)
            .map_err(|error| NluRuntimeError::Tokenizer(error.to_string()))?;
        if encoding.len() > self.contract.max_length {
            encoding.truncate(self.contract.max_length, 0, TruncationDirection::Right);
        }

        let input_ids = encoding
            .get_ids()
            .iter()
            .map(|value| i64::from(*value))
            .collect::<Vec<_>>();

        let attention_mask = encoding
            .get_attention_mask()
            .iter()
            .map(|value| i64::from(*value))
            .collect::<Vec<_>>();

        let input_ids = Array2::from_shape_vec((1, input_ids.len()), input_ids)
            .map_err(|error| NluRuntimeError::InvalidArtifact(error.to_string()))?;

        let attention_mask = Array2::from_shape_vec((1, attention_mask.len()), attention_mask)
            .map_err(|error| NluRuntimeError::InvalidArtifact(error.to_string()))?;

        Ok(EncodedInput {
            input_ids,
            attention_mask,
            tokens: encoding.get_tokens().to_vec(),
            offsets: encoding.get_offsets().to_vec(),
        })
    }

    fn run_model(&self, input: &EncodedInput) -> Result<OnnxModelOutput, NluRuntimeError> {
        let mut session = self
            .session
            .lock()
            .map_err(|_| NluRuntimeError::Onnx("failed to lock session".to_string()))?;

        let outputs = session
            .run(ort::inputs! {
                "input_ids" => TensorRef::from_array_view(input.input_ids.view()).map_err(|error| NluRuntimeError::Onnx(error.to_string()))?,
                "attention_mask" => TensorRef::from_array_view(input.attention_mask.view()).map_err(|error| NluRuntimeError::Onnx(error.to_string()))?,
            })
            .map_err(|error| NluRuntimeError::Onnx(error.to_string()))?;

        let (_, intent_values) = outputs["intent_logits"]
            .try_extract_tensor::<f32>()
            .map_err(|error| NluRuntimeError::Onnx(error.to_string()))?;

        let (_, ner_values) = outputs["ner_logits"]
            .try_extract_tensor::<f32>()
            .map_err(|error| NluRuntimeError::Onnx(error.to_string()))?;

        Ok(OnnxModelOutput {
            intent_logits: intent_values.to_vec(),
            ner_logits: ner_values.to_vec(),
        })
    }
}

impl NluModelRuntime for OnnxNluRuntime {
    fn contract(&self) -> &OnnxContract {
        &self.contract
    }

    fn label_maps(&self) -> &LabelMaps {
        &self.label_maps
    }

    fn run(&self, input: TaggedInput) -> Result<NluModelInference, NluRuntimeError> {
        let input = self.encode_input(input)?;
        let outputs = self.run_model(&input)?;
        Ok(NluModelInference {
            tokens: input.tokens,
            offsets: input.offsets,
            outputs,
        })
    }
}
