use crate::core::conversation::domain::model::slot::EntityType;

/// Runtime context attached to an utterance before model inference.
#[derive(Debug, Clone, PartialEq)]
pub struct InferenceContext {
    pub lang: String,
    pub domain: String,
    pub task: Option<String>,
}

/// Preprocessed input string sent to the model, with the prefix length needed to
/// map token offsets back to the raw user text.
#[derive(Debug, Clone, PartialEq)]
pub struct TaggedInput {
    pub text: String,
    pub prefix_length: usize,
}

/// Primary intent selected from the model output.
#[derive(Debug, Clone, PartialEq)]
pub struct NluIntent {
    pub name: String,
    pub confidence: f32,
}

/// Ranked intent candidate with its probability-like confidence.
#[derive(Debug, Clone, PartialEq)]
pub struct NluIntentCandidate {
    pub name: String,
    pub confidence: f32,
}

/// Decoded entity span mapped back to the raw user text.
#[derive(Debug, Clone, PartialEq)]
pub struct NluEntity {
    pub entity_type: EntityType,
    pub value: String,
    pub raw_value: String,
    pub start: usize,
    pub end: usize,
    pub confidence: f32,
}

/// Token-level NER label kept for debugging and observability.
#[derive(Debug, Clone, PartialEq)]
pub struct NerTokenLabel {
    pub token: String,
    pub label: String,
    pub start: usize,
    pub end: usize,
}

/// Final NLU result returned to callers of the NLU engine.
#[derive(Debug, Clone, PartialEq)]
pub struct NluAnalysis {
    pub processed_text: String,
    pub intent: NluIntent,
    pub intents: Vec<NluIntentCandidate>,
    pub entities: Vec<NluEntity>,
    pub ner_labels: Vec<NerTokenLabel>,
}

impl InferenceContext {
    /// Creates the inference context consumed by tagged-input preprocessing.
    pub fn new(lang: impl Into<String>, domain: impl Into<String>, task: Option<String>) -> Self {
        Self {
            lang: lang.into(),
            domain: domain.into(),
            task,
        }
    }
}

impl TaggedInput {
    /// Builds the tagged model input from the raw text and inference context.
    ///
    /// The tag order must stay aligned with `model_training`:
    /// optional task, then language, then domain, then raw text.
    pub fn build(text: &str, context: &InferenceContext) -> Self {
        let mut tags = Vec::new();
        if let Some(task) = context.task.as_ref() {
            tags.push(format!("[TASK={task}]"));
        }
        tags.push(format!("[LANG={}]", context.lang));
        tags.push(format!("[DOMAIN={}]", context.domain));
        let prefix = tags.join(" ");
        Self {
            text: format!("{prefix} {text}"),
            prefix_length: prefix.len() + 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tagged_input_omits_task_when_missing() {
        let context = InferenceContext::new("en", "restaurant", None);
        let tagged = TaggedInput::build("Hello", &context);
        assert_eq!(tagged.text, "[LANG=en] [DOMAIN=restaurant] Hello");
    }

    #[test]
    fn tagged_input_includes_task_when_present() {
        let context = InferenceContext::new(
            "id",
            "restaurant",
            Some("WF_RESERVATION_CREATE".to_string()),
        );
        let tagged = TaggedInput::build("empat orang", &context);
        assert_eq!(
            tagged.text,
            "[TASK=WF_RESERVATION_CREATE] [LANG=id] [DOMAIN=restaurant] empat orang"
        );
    }
}
