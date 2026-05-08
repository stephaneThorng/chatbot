use crate::core::nlu_engine::domain::analysis::TaggedInput;

#[derive(Debug, Clone, PartialEq)]
pub struct NluInferenceInput {
    pub tagged_input: TaggedInput,
}

impl NluInferenceInput {
    pub fn new(tagged_input: TaggedInput) -> Self {
        Self { tagged_input }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OnnxModelOutput {
    pub intent_logits: Vec<f32>,
    pub ner_logits: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NluModelInference {
    pub tokens: Vec<String>,
    pub offsets: Vec<(usize, usize)>,
    pub outputs: OnnxModelOutput,
}
