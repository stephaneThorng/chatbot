/// Conversation-owned representation of a completed NLU analysis.
///
/// This DTO decouples the `conversation` application layer from
/// `nlu_engine::domain` types. The outbound gateway adapter is responsible
/// for mapping `NluAnalysis → NluAnalysisResult`.
#[derive(Debug, PartialEq)]
pub struct NluAnalysisResult {
    pub intent_name: String,
    pub intent_confidence: f32,
    pub intent_candidates: Vec<NluIntentCandidate>,
    pub entities: Vec<NluEntityResult>,
}

/// Ranked intent candidate carried by `NluAnalysisResult`.
#[derive(Debug, PartialEq)]
pub struct NluIntentCandidate {
    pub name: String,
    pub confidence: f32,
}

/// Decoded entity span carried by `NluAnalysisResult`.
#[derive(Debug, PartialEq)]
pub struct NluEntityResult {
    /// The NLU entity label string, e.g. `"person"`, `"date"`.
    pub entity_label: String,
    pub value: String,
    pub raw_value: String,
    pub start: usize,
    pub end: usize,
    pub confidence: f32,
}
