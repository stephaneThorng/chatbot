/// Raw ONNX logits returned by the runtime before application-layer decoding.
#[derive(Debug, Clone, PartialEq)]
pub struct OnnxModelOutput {
    pub intent_logits: Vec<f32>,
    pub ner_logits: Vec<f32>,
}

/// Tokenizer metadata and raw model outputs produced by the runtime port.
#[derive(Debug, Clone, PartialEq)]
pub struct NluModelInference {
    pub tokens: Vec<String>,
    pub offsets: Vec<(usize, usize)>,
    pub outputs: OnnxModelOutput,
}
