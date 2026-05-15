use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct LabelMaps {
    pub intent_label2id: std::collections::HashMap<String, usize>,
    pub intent_id2label: std::collections::HashMap<String, String>,
    pub ner_label2id: std::collections::HashMap<String, usize>,
    pub ner_id2label: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct OnnxContract {
    pub max_length: usize,
    pub model_inputs: Vec<String>,
    pub model_outputs: Vec<String>,
    pub labels: ContractLabels,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ContractLabels {
    pub intents: Vec<String>,
    pub ner: Vec<String>,
}
